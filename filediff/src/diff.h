#pragma once

#include <string>
#include <vector>

struct DiffLine {
    enum Type { Added, Removed, Context } type;
    std::string content;
    int lineNum;
};

class Diff {
public:
    std::vector<DiffLine> compute(const std::vector<std::string>& oldLines,
                                   const std::vector<std::string>& newLines);

private:
    std::vector<std::vector<int>> computeLCS(const std::vector<std::string>& a,
                                              const std::vector<std::string>& b);
};
