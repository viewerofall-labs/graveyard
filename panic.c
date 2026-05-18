#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <signal.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>

volatile sig_atomic_t timeout_expired = 0;

void timeout_handler(int sig __attribute__((unused))) {
    timeout_expired = 1;
}

int main(void) {
    struct sigaction sa;
    char response[10];
    int fd;

    printf("╔═══════════════════════════════════════════╗\n");
    printf("║     KERNEL PANIC TRIGGER - NUCLEAR MODE   ║\n");
    printf("╚═══════════════════════════════════════════╝\n\n");
    printf("⚠️  This will immediately crash your system.\n");
    printf("⚠️  All unsaved data will be LOST.\n\n");

    printf("Confirm? (yes/no) [10 second timeout]: ");
    fflush(stdout);

    // Set up 10-second alarm
    memset(&sa, 0, sizeof(sa));
    sa.sa_handler = timeout_handler;
    sigaction(SIGALRM, &sa, NULL);
    alarm(10);

    // Read user input
    if (fgets(response, sizeof(response), stdin) == NULL) {
        printf("\nNo input received. Aborting.\n");
        alarm(0);
        return 0;
    }

    alarm(0); // Cancel alarm

    // Check response
    if (strncasecmp(response, "yes", 3) != 0) {
        printf("\nAborted.\n");
        return 0;
    }

    printf("\n⏱️  Initiating panic sequence...\n");
    sleep(1);

    // Trigger kernel panic via SysRq
    fd = open("/proc/sysrq-trigger", O_WRONLY);
    if (fd < 0) {
        printf("ERROR: Cannot open /proc/sysrq-trigger.\n");
        printf("Make sure kernel.sysrq is enabled:\n");
        printf("  sudo sysctl kernel.sysrq=1\n");
        return 1;
    }

    if (write(fd, "c", 1) < 0) {
        printf("ERROR: Failed to write to /proc/sysrq-trigger.\n");
        close(fd);
        return 1;
    }

    close(fd);

    // Should never reach here
    printf("Panic command sent. System should crash now...\n");
    sleep(5); // Fallback delay in case it doesn't
    return 0;
}
