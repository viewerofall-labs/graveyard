#include "ui.h"
#include "imgui.h"
#include <fstream>
#include <sstream>
#include <cstring>

void FileEditor::loadFromFile(const std::string& path) {
    std::ifstream file(path);
    if (!file.is_open()) return;

    std::stringstream buffer;
    buffer << file.rdbuf();
    text = buffer.str();
    filename = path;
    isModified = false;
}

void FileEditor::setText(const std::string& content) {
    text = content;
    isModified = true;
}

std::vector<std::string> FileEditor::getLines() const {
    std::vector<std::string> lines;
    std::stringstream ss(text);
    std::string line;
    while (std::getline(ss, line)) {
        lines.push_back(line);
    }
    return lines;
}

void DiffViewer::computeDiff() {
    Diff d;
    diffResult = d.compute(left.getLines(), right.getLines());
    showDiff = true;
}

void DiffViewer::renderEditor(FileEditor& editor, const char* label, char* buf, char* filePath) {
    ImGui::Text("%s", label);
    if (ImGui::Button(("Load##" + std::string(label)).c_str(), ImVec2(60, 0))) {
        if (filePath[0] != '\0') {
            editor.loadFromFile(filePath);
            strncpy(buf, editor.text.c_str(), BUFFER_SIZE - 1);
        }
    }
    ImGui::SameLine();
    ImGui::InputText(("File##" + std::string(label)).c_str(), filePath, 512);

    if (ImGui::InputTextMultiline(("##text" + std::string(label)).c_str(), buf, BUFFER_SIZE,
                                   ImVec2(-1, 200), ImGuiInputTextFlags_AllowTabInput)) {
        editor.setText(buf);
    }
}

void DiffViewer::renderDiffOutput() {
    if (ImGui::BeginChild("DiffOutput", ImVec2(0, -30), true)) {
        for (const auto& line : diffResult) {
            ImVec4 color;
            std::string prefix;

            switch (line.type) {
                case DiffLine::Added:
                    color = ImVec4(0.2f, 0.8f, 0.2f, 1.0f);
                    prefix = "+ ";
                    break;
                case DiffLine::Removed:
                    color = ImVec4(0.8f, 0.2f, 0.2f, 1.0f);
                    prefix = "- ";
                    break;
                case DiffLine::Context:
                    color = ImVec4(0.8f, 0.8f, 0.8f, 1.0f);
                    prefix = "  ";
                    break;
            }

            ImGui::PushStyleColor(ImGuiCol_Text, color);
            ImGui::TextUnformatted((prefix + line.content).c_str());
            ImGui::PopStyleColor();
        }
        ImGui::EndChild();
    }
}

void DiffViewer::render() {
    ImGui::SetNextWindowPos(ImVec2(100, 100), ImGuiCond_FirstUseEver);
    ImGui::SetNextWindowSize(ImVec2(1000, 700), ImGuiCond_FirstUseEver);

    ImGui::Begin("filediff", nullptr, ImGuiWindowFlags_NoMove);

    ImGui::Text("File Diff");
    ImGui::Separator();

    if (ImGui::BeginTable("Editors", 2, ImGuiTableFlags_Resizable | ImGuiTableFlags_BordersV)) {
        ImGui::TableSetupColumn("Old", ImGuiTableColumnFlags_WidthStretch);
        ImGui::TableSetupColumn("New", ImGuiTableColumnFlags_WidthStretch);

        ImGui::TableNextRow();
        ImGui::TableSetColumnIndex(0);
        renderEditor(left, "Left", leftBuf, leftFilePath);

        ImGui::TableSetColumnIndex(1);
        renderEditor(right, "Right", rightBuf, rightFilePath);

        ImGui::EndTable();
    }

    ImGui::Separator();
    if (ImGui::Button("Compare", ImVec2(80, 0))) {
        strncpy(leftBuf, left.text.c_str(), BUFFER_SIZE - 1);
        strncpy(rightBuf, right.text.c_str(), BUFFER_SIZE - 1);
        computeDiff();
    }

    ImGui::SameLine();
    ImGui::Checkbox("Show Diff", &showDiff);

    if (showDiff) {
        renderDiffOutput();
    }

    ImGui::End();
}
