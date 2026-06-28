#pragma once

#include "diff.h"
#include <string>
#include <vector>

class FileEditor {
public:
    std::string text;
    std::string filename;
    bool isModified = false;

    void loadFromFile(const std::string& path);
    void setText(const std::string& content);
    std::vector<std::string> getLines() const;
};

class DiffViewer {
public:
    FileEditor left, right;
    std::vector<DiffLine> diffResult;
    bool showDiff = false;

    void computeDiff();
    void render();

private:
    static constexpr int BUFFER_SIZE = 65536;
    char leftBuf[BUFFER_SIZE] = {0};
    char rightBuf[BUFFER_SIZE] = {0};
    char leftFilePath[512] = {0};
    char rightFilePath[512] = {0};

    void renderEditor(FileEditor& editor, const char* label, char* buf, char* filePath);
    void renderDiffOutput();
};
