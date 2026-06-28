#include "diff.h"
#include <algorithm>

std::vector<std::vector<int>> Diff::computeLCS(const std::vector<std::string>& a,
                                                const std::vector<std::string>& b) {
    int m = a.size();
    int n = b.size();
    std::vector<std::vector<int>> dp(m + 1, std::vector<int>(n + 1, 0));

    for (int i = 1; i <= m; ++i) {
        for (int j = 1; j <= n; ++j) {
            if (a[i - 1] == b[j - 1]) {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = std::max(dp[i - 1][j], dp[i][j - 1]);
            }
        }
    }
    return dp;
}

std::vector<DiffLine> Diff::compute(const std::vector<std::string>& oldLines,
                                     const std::vector<std::string>& newLines) {
    std::vector<DiffLine> result;
    auto dp = computeLCS(oldLines, newLines);

    int i = oldLines.size();
    int j = newLines.size();

    while (i > 0 || j > 0) {
        if (i > 0 && j > 0 && oldLines[i - 1] == newLines[j - 1]) {
            result.push_back({DiffLine::Context, oldLines[i - 1], i});
            --i;
            --j;
        } else if (j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j])) {
            result.push_back({DiffLine::Added, newLines[j - 1], j});
            --j;
        } else if (i > 0) {
            result.push_back({DiffLine::Removed, oldLines[i - 1], i});
            --i;
        }
    }

    std::reverse(result.begin(), result.end());
    return result;
}
