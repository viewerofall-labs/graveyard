#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <libgen.h>
#include <unistd.h>
#include <limits.h>

// Helper to find the extension (returns pointer to the '.' or NULL)
char *get_extension(char *filename) {
    char *dot = strrchr(filename, '.');
    if (!dot || dot == filename) return NULL;
    return dot;
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <path/to/file> [new_name]\n", argv[0]);
        return 1;
    }

    char *filepath = argv[1];
    char *old_base = basename(strdup(filepath));
    char *old_ext = get_extension(old_base);
    char input_name[NAME_MAX];
    char final_name[NAME_MAX];
    char new_path[PATH_MAX];

    // 1. Get the new name (from arg or prompt)
    if (argc >= 3) {
        strncpy(input_name, argv[2], NAME_MAX);
    } else {
        printf("Rename '%s' to: ", old_base);
        if (fgets(input_name, sizeof(input_name), stdin) == NULL) return 1;
        input_name[strcspn(input_name, "\n")] = 0;
    }

    if (strlen(input_name) == 0) return 1;

    // 2. Extension preservation logic
    // If input has no dot AND the original file DID have an extension...
    if (strrchr(input_name, '.') == NULL && old_ext != NULL) {
        snprintf(final_name, sizeof(final_name), "%s%s", input_name, old_ext);
    } else {
        // Either user provided an extension or original had none
        strncpy(final_name, input_name, NAME_MAX);
    }

    // 3. Construct full path
    char *path_copy = strdup(filepath);
    char *dir = dirname(path_copy);
    snprintf(new_path, sizeof(new_path), "%s/%s", dir, final_name);

    // 4. Atomic Rename
    if (rename(filepath, new_path) == 0) {
        printf("Renamed: %s -> %s\n", old_base, final_name);
    } else {
        perror("Error");
        return 1;
    }

    return 0;
}
