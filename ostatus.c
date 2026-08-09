#include <stdio.h>
#include <time.h>
#include <sys/stat.h>

int main(void) {
    struct stat root_stat;
    time_t now;
    int days, hours, mins, secs;
    long total_secs;

    /* Get root filesystem stats */
    if (stat("/", &root_stat) == -1) {
        perror("stat");
        return 1;
    }

    /* Get current time */
    now = time(NULL);

    /* Calculate difference in seconds */
    total_secs = (long)difftime(now, root_stat.st_ctime);

    if (total_secs < 0) {
        fprintf(stderr, "Error: filesystem time in future\n");
        return 1;
    }

    /* Break down into days, hours, mins, secs */
    days = total_secs / 86400;
    hours = (total_secs % 86400) / 3600;
    mins = (total_secs % 3600) / 60;
    secs = total_secs % 60;

    printf("OS Created: %s", ctime(&root_stat.st_ctime));
    printf("OS Age: %d days, %d hours, %d minutes, %d seconds\n", days, hours, mins, secs);

    return 0;
}
