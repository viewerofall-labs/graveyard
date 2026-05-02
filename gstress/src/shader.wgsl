@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= arrayLength(&input)) {
        return;
    }
    
    var result = input[idx];
    
    // Heavy mathematical operations to stress the GPU
    for (var i = 0u; i < 100u; i++) {
        result = sin(result) * cos(result) + tan(result * 0.01);
        result = sqrt(abs(result)) + pow(abs(result), 1.5);
        result = result * 1.01 - 0.001;
        
        // Matrix-like operations
        for (var j = 0u; j < 10u; j++) {
            let val = f32(j) * 0.1;
            result = result + sin(val * result) * cos(val);
        }
    }
    
    output[idx] = result;
}
