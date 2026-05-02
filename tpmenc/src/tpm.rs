use anyhow::{Context as _, Result};
use std::path::{Path, PathBuf};
use tss_esapi::{
    Context, TctiNameConf,
    attributes::ObjectAttributesBuilder,
    interface_types::{
        algorithm::{HashingAlgorithm, PublicAlgorithm},
        resource_handles::Hierarchy,
    },
    structures::{
        Digest, KeyedHashScheme, Private, Public, PublicBuilder, PublicBuffer,
        PublicKeyedHashParameters, SensitiveData, SymmetricCipherParameters,
        SymmetricDefinitionObject,
    },
};

use crate::crypto;

// File format:
//   [4 bytes LE] public_len
//   [public_len] marshalled TPM Public (PublicBuffer bytes)
//   [4 bytes LE] private_len
//   [private_len] Private bytes
//   [12 bytes]   AES-GCM nonce
//   [rest]       AES-GCM ciphertext + tag

fn open_context() -> Result<Context> {
    let tcti = TctiNameConf::from_environment_variable()
        .unwrap_or_else(|_| "device:/dev/tpmrm0".parse().expect("bad tcti string"));
    Context::new(tcti).context("failed to open TPM context — is tpm2-tss installed and /dev/tpmrm0 accessible?")
}

fn make_primary(ctx: &mut Context) -> Result<tss_esapi::structures::CreatePrimaryKeyResult> {
    let attrs = ObjectAttributesBuilder::new()
        .with_fixed_tpm(true)
        .with_fixed_parent(true)
        .with_st_clear(false)
        .with_sensitive_data_origin(true)
        .with_user_with_auth(true)
        .with_decrypt(true)
        .with_restricted(true)
        .build()
        .context("build srk attrs")?;

    let pub_template = PublicBuilder::new()
        .with_public_algorithm(PublicAlgorithm::SymCipher)
        .with_name_hashing_algorithm(HashingAlgorithm::Sha256)
        .with_object_attributes(attrs)
        .with_symmetric_cipher_parameters(SymmetricCipherParameters::new(
            SymmetricDefinitionObject::AES_128_CFB,
        ))
        .with_symmetric_cipher_unique_identifier(Digest::default())
        .build()
        .context("build srk public")?;

    ctx.execute_with_nullauth_session(|ctx| {
        ctx.create_primary(Hierarchy::Owner, pub_template, None, None, None, None)
    })
    .context("create_primary failed")
}

fn sealed_object_template() -> Result<Public> {
    let attrs = ObjectAttributesBuilder::new()
        .with_fixed_tpm(true)
        .with_fixed_parent(true)
        .with_st_clear(true)
        .with_user_with_auth(true)
        .build()
        .context("build sealed attrs")?;

    PublicBuilder::new()
        .with_public_algorithm(PublicAlgorithm::KeyedHash)
        .with_name_hashing_algorithm(HashingAlgorithm::Sha256)
        .with_object_attributes(attrs)
        .with_keyed_hash_parameters(PublicKeyedHashParameters::new(KeyedHashScheme::Null))
        .with_keyed_hash_unique_identifier(Digest::default())
        .build()
        .context("build sealed public template")
}

pub fn seal_file(input: &Path) -> Result<PathBuf> {
    let plaintext = std::fs::read(input).with_context(|| format!("read {}", input.display()))?;

    // Generate a random 32-byte AES key and encrypt the file
    let (nonce, ciphertext, aes_key) = crypto::encrypt(&plaintext)?;

    // Seal the AES key into the TPM
    let mut ctx = open_context()?;
    let primary = make_primary(&mut ctx)?;
    let template = sealed_object_template()?;

    let sensitive = SensitiveData::try_from(aes_key.to_vec()).context("sensitive data")?;

    let (tpm_private, tpm_public) = ctx
        .execute_with_nullauth_session(|ctx| {
            ctx.create(primary.key_handle, template, None, Some(sensitive), None, None)
                .map(|r| (r.out_private, r.out_public))
        })
        .context("TPM create sealed object failed")?;

    ctx.flush_context(primary.key_handle.into())
        .context("flush primary")?;

    // Serialise the TPM blobs
    let pub_bytes = PublicBuffer::try_from(tpm_public)
        .context("serialise Public")?
        .value()
        .to_vec();
    let priv_bytes = tpm_private.value().to_vec();

    // Write sealed file
    let mut out_data: Vec<u8> = Vec::new();
    out_data.extend_from_slice(&(pub_bytes.len() as u32).to_le_bytes());
    out_data.extend_from_slice(&pub_bytes);
    out_data.extend_from_slice(&(priv_bytes.len() as u32).to_le_bytes());
    out_data.extend_from_slice(&priv_bytes);
    out_data.extend_from_slice(&nonce);
    out_data.extend_from_slice(&ciphertext);

    let out_path = input.with_extension(
        input
            .extension()
            .map(|e| format!("{}.sealed", e.to_string_lossy()))
            .unwrap_or_else(|| "sealed".to_string()),
    );
    std::fs::write(&out_path, &out_data)
        .with_context(|| format!("write {}", out_path.display()))?;

    Ok(out_path)
}

fn decrypt_sealed(input: &Path) -> Result<Vec<u8>> {
    let data = std::fs::read(input).with_context(|| format!("read {}", input.display()))?;
    let mut cursor = 0usize;

    macro_rules! read_u32 {
        () => {{
            if cursor + 4 > data.len() {
                anyhow::bail!("sealed file truncated");
            }
            let n = u32::from_le_bytes(data[cursor..cursor + 4].try_into().unwrap()) as usize;
            cursor += 4;
            n
        }};
    }
    macro_rules! read_bytes {
        ($n:expr) => {{
            let n = $n;
            if cursor + n > data.len() {
                anyhow::bail!("sealed file truncated");
            }
            let s = &data[cursor..cursor + n];
            cursor += n;
            s
        }};
    }

    let pub_len = read_u32!();
    let pub_bytes = read_bytes!(pub_len).to_vec();
    let priv_len = read_u32!();
    let priv_bytes = read_bytes!(priv_len).to_vec();
    let nonce: [u8; 12] = read_bytes!(12).try_into().context("nonce size")?;
    let ciphertext = data[cursor..].to_vec();

    // Reconstruct TPM objects
    let tpm_public =
        Public::try_from(PublicBuffer::try_from(pub_bytes).context("deserialise Public")?)
            .context("Public from buffer")?;
    let tpm_private = Private::try_from(priv_bytes.as_slice()).context("deserialise Private")?;

    // Unseal AES key from TPM
    let mut ctx = open_context()?;
    let primary = make_primary(&mut ctx)?;

    let aes_key: [u8; 32] = ctx
        .execute_with_nullauth_session(|ctx| -> anyhow::Result<[u8; 32]> {
            let handle = ctx
                .load(primary.key_handle, tpm_private, tpm_public)
                .context("load sealed object")?;
            let sensitive = ctx.unseal(handle.into()).context("unseal")?;
            sensitive
                .value()
                .try_into()
                .map_err(|_| anyhow::anyhow!("unexpected key length: expected 32 bytes"))
        })
        .context("TPM unseal failed")?;

    ctx.flush_context(primary.key_handle.into())
        .context("flush primary")?;

    crypto::decrypt(&nonce, &ciphertext, &aes_key)
}

pub fn preview_file(input: &Path) -> Result<Vec<u8>> {
    decrypt_sealed(input)
}

pub fn unseal_file(input: &Path) -> Result<PathBuf> {
    let plaintext = decrypt_sealed(input)?;

    // Strip .sealed (handles both "file.txt.sealed" and "file.sealed")
    let out_path = {
        let s = input.to_string_lossy();
        let stripped = s
            .strip_suffix(".sealed")
            .unwrap_or_else(|| s.trim_end_matches(".sealed"));
        PathBuf::from(stripped.to_string())
    };
    std::fs::write(&out_path, &plaintext)
        .with_context(|| format!("write {}", out_path.display()))?;

    Ok(out_path)
}
