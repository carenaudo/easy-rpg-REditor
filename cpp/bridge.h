#pragma once
#include <string>
#include <vector>

std::vector<std::string> load_project(const std::string& path);
std::vector<uint8_t> get_map_chipset(const std::string& path, int map_id);
