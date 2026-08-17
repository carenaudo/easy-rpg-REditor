#include "bridge.h"
#include <lcf/ldb/reader.h>
#include <lcf/lmt/reader.h>
#include <lcf/lmu/reader.h>
#include <lcf/data.h>
#include <fstream>

std::vector<std::string> load_project(const std::string& path) {
    std::vector<std::string> maps;

    if (!lcf::LMT_Reader::Load(path + "/RPG_RT.lmt", "UTF-8")) {
        maps.push_back("[Error] Failed to load project");
        return maps;
    }

    for (const auto& map : lcf::Data::treemap.maps) {
        maps.push_back(map.name);
    }
    return maps;
}

std::vector<uint8_t> get_map_chipset(const std::string& path, int map_id) {
    std::vector<uint8_t> data;

    if (!lcf::LMU_Reader::Load(path + "/Map" + std::to_string(map_id) + ".lmu", "UTF-8")) {
        return data;
    }

    int chipset_id = lcf::Data::map.chipset_id;
    if (chipset_id <= 0 || chipset_id > (int)lcf::Data::chipsets.size()) {
        return data;
    }

    std::string chipset_name = lcf::Data::chipsets[chipset_id - 1].chipset_name;
    std::string file_path = path + "/" + chipset_name + ".png";

    std::ifstream file(file_path, std::ios::binary);
    if (!file) return data;

    file.seekg(0, std::ios::end);
    size_t size = file.tellg();
    file.seekg(0, std::ios::beg);

    data.resize(size);
    file.read(reinterpret_cast<char*>(data.data()), size);
    return data;
}
