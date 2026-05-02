#include <unistd.h>
#include <stdio.h>
#include <sys/stat.h>
#include <string.h>

int main(int argc, char *argv[]) {
    struct stat path_stat;
    char *target = "."; // Default to current directory if no path is given
    int is_file = 0;    // Default to assuming directory behavior

    // Step 1: Scan arguments to find the target path
    // We look for the first argument that doesn't start with a '-' (flag)
    for (int i = 1; i < argc; i++) {
        if (argv[i][0] != '-') {
            target = argv[i];
            break;
        }
    }

    // Step 2: Use stat() to check the filesystem
    // If stat returns 0 (success), we check the mode
    if (stat(target, &path_stat) == 0) {
        if (S_ISREG(path_stat.st_mode)) {
            is_file = 1; // It is a regular file!
        }
    }

    // Step 3: Route to the appropriate binary
    if (is_file) {
        // Change the 0th argument to the new program name (good practice)
        argv[0] = "bat"; 
        
        // execvp searches your $PATH for 'bat'
        execvp("bat", argv);
        
        // This only runs if bat is missing
        perror("Error: Could not execute bat"); 
    } else {
        argv[0] = "lsd";
        execvp("lsd", argv);

        perror("Error: Could not execute lsd");
    }

    return 1;
}
