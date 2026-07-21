#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <unistd.h>
#include <dirent.h>

// Parse time string like "5s", "10m", "2h" into total seconds
int parse_duration(const char *str) {
    char *endptr;
    long value = strtol(str, &endptr, 10);
    if (endptr == str || value <= 0) return -1;
    
    char unit = *endptr ? tolower((unsigned char)*endptr) : 's';
    
    if (unit == 's') return value;
    if (unit == 'm') return value * 60;
    if (unit == 'h') return value * 3600;
    return -1;
}

// Directly gets the VmRSS (RAM) for a single explicit PID
unsigned long get_ram_by_pid(int pid) {
    char path[256];
    snprintf(path, sizeof(path), "/proc/%d/status", pid);
    FILE *fstat = fopen(path, "r");
    if (!fstat) return 0; // Process might have died

    char line[256];
    unsigned long rss_kb = 0;
    while (fgets(line, sizeof(line), fstat)) {
        if (strncmp(line, "VmRSS:", 6) == 0) {
            sscanf(line + 6, "%lu", &rss_kb);
            break;
        }
    }
    fclose(fstat);
    return rss_kb;
}

// Scans /proc to find all matching processes by full command line (fallback mode)
unsigned long get_program_ram_by_name(const char *full_target_cmd) {
    DIR *dir = opendir("/proc");
    if (!dir) return 0;

    struct dirent *entry;
    unsigned long total_rss_kb = 0;

    while ((entry = readdir(dir)) != NULL) {
        if (!isdigit((unsigned char)entry->d_name[0])) continue;

        char path[512];
        snprintf(path, sizeof(path), "/proc/%s/cmdline", entry->d_name);
        
        FILE *fcmd = fopen(path, "rb");
        if (!fcmd) continue;

        char raw_cmd[2048];
        size_t bytes_read = fread(raw_cmd, 1, sizeof(raw_cmd) - 1, fcmd);
        fclose(fcmd);

        if (bytes_read <= 0) continue;
        raw_cmd[bytes_read] = '\0';

        char full_running_cmd[2048] = {0};
        size_t current_pos = 0;
        
        for (size_t i = 0; i < bytes_read; i++) {
            if (raw_cmd[i] == '\0') {
                if (i + 1 < bytes_read && raw_cmd[i + 1] != '\0') {
                    if (current_pos < sizeof(full_running_cmd) - 1) {
                        full_running_cmd[current_pos++] = ' ';
                    }
                }
            } else {
                if (current_pos < sizeof(full_running_cmd) - 1) {
                    full_running_cmd[current_pos++] = raw_cmd[i];
                }
            }
        }
        full_running_cmd[current_pos] = '\0';

        if (strstr(full_running_cmd, full_target_cmd) != NULL) {
            int pid = atoi(entry->d_name);
            total_rss_kb += get_ram_by_pid(pid);
        }
    }
    closedir(dir);
    return total_rss_kb;
}

int main(int argc, char *argv[]) {
    int total_seconds = 10;
    int target_pid = -1;
    const char *target_program = NULL;
    int is_pid_mode = 0;

    // Parse command line arguments
    int arg_idx = 1;

    if (arg_idx < argc && strcmp(argv[arg_idx], "-p") == 0) {
        is_pid_mode = 1;
        arg_idx++;
        if (arg_idx >= argc) {
            fprintf(stderr, "Error: Missing PID after -p\n");
            return 1;
        }
        target_pid = atoi(argv[arg_idx]);
        arg_idx++;
    } else if (arg_idx + 1 < argc && strcmp(argv[arg_idx + 1], "-p") == 0) {
        // Handle case where duration comes BEFORE -p (e.g., ./rampgm 5s -p 1234)
        total_seconds = parse_duration(argv[arg_idx]);
        if (total_seconds <= 0) {
            fprintf(stderr, "Error: Invalid duration format '%s'.\n", argv[arg_idx]);
            return 1;
        }
        is_pid_mode = 1;
        arg_idx += 2; // skip duration and "-p"
        if (arg_idx >= argc) {
            fprintf(stderr, "Error: Missing PID after -p\n");
            return 1;
        }
        target_pid = atoi(argv[arg_idx]);
        arg_idx++;
    }

    // If we aren't in PID mode, parse using original logic
    if (!is_pid_mode) {
        if (argc == 2) {
            target_program = argv[1];
        } else if (argc == 3) {
            total_seconds = parse_duration(argv[1]);
            if (total_seconds <= 0) {
                total_seconds = 10;
                target_program = argv[1];
            } else {
                target_program = argv[2];
            }
        } else {
            fprintf(stderr, "Usage:\n");
            fprintf(stderr, "  By Name: %s [duration] \"<program_name>\"\n", argv[0]);
            fprintf(stderr, "  By PID : %s [duration] -p <pid>\n", argv[0]);
            return 1;
        }
    }

    if (is_pid_mode) {
        printf("Monitoring PID %d for %d seconds...\n", target_pid, total_seconds);
    } else {
        printf("Monitoring targets matching: '%s' for %d seconds...\n", target_program, total_seconds);
    }
    printf("----------------------------------------\n");

    unsigned long long sum_ram = 0;
    int samples_taken = 0;

    for (int i = 1; i <= total_seconds; i++) {
        sleep(1); 
        unsigned long current_ram_kb = 0;
        
        if (is_pid_mode) {
            current_ram_kb = get_ram_by_pid(target_pid);
            if (current_ram_kb == 0 && samples_taken > 0) {
                printf("Process %d terminated early.\n", target_pid);
                break;
            }
        } else {
            current_ram_kb = get_program_ram_by_name(target_program);
        }

        double current_ram_mb = current_ram_kb / 1024.0;
        printf("Second %d: %.2f MB\n", i, current_ram_mb);
        
        sum_ram += current_ram_kb;
        samples_taken++;
    }

    printf("----------------------------------------\n");
    if (samples_taken > 0) {
        double avg_ram_mb = (sum_ram / (double)samples_taken) / 1024.0;
        printf("Average RAM Usage: %.2f MB\n", avg_ram_mb);
    } else {
        printf("No data collected.\n");
    }

    return 0;
}