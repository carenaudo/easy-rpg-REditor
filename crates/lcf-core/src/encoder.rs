use encoding_rs::Encoding;

#[derive(Debug, Clone)]
pub struct LcfEncoder {
    encoding_name: String,
    encoding: &'static Encoding,
    is_auto: bool,
}

impl Default for LcfEncoder {
    fn default() -> Self {
        Self::new("auto")
    }
}

impl LcfEncoder {
    pub fn new(encoding_str: &str) -> Self {
        let trimmed = encoding_str.trim();
        let is_auto = trimmed.is_empty() || trimmed.eq_ignore_ascii_case("auto");
        let enc = Self::resolve_encoding(trimmed);
        Self {
            encoding_name: encoding_str.to_string(),
            encoding: enc,
            is_auto,
        }
    }

    fn resolve_encoding(name: &str) -> &'static Encoding {
        if name.is_empty() || name.eq_ignore_ascii_case("auto") || name.eq_ignore_ascii_case("utf-8") || name.eq_ignore_ascii_case("utf8") {
            return encoding_rs::UTF_8;
        }

        // Check common aliases
        let lower = name.to_ascii_lowercase();
        match lower.as_str() {
            "sjis" | "shift_jis" | "shift-jis" | "cp932" | "932" | "ms932" | "windows-31j" => encoding_rs::SHIFT_JIS,
            "1252" | "cp1252" | "windows-1252" | "win1252" | "latin1" | "iso-8859-1" => encoding_rs::WINDOWS_1252,
            "1250" | "cp1250" | "windows-1250" => encoding_rs::WINDOWS_1250,
            "1251" | "cp1251" | "windows-1251" => encoding_rs::WINDOWS_1251,
            "1253" | "cp1253" | "windows-1253" => encoding_rs::WINDOWS_1253,
            "1254" | "cp1254" | "windows-1254" => encoding_rs::WINDOWS_1254,
            "1255" | "cp1255" | "windows-1255" => encoding_rs::WINDOWS_1255,
            "1256" | "cp1256" | "windows-1256" => encoding_rs::WINDOWS_1256,
            "1257" | "cp1257" | "windows-1257" => encoding_rs::WINDOWS_1257,
            "1258" | "cp1258" | "windows-1258" => encoding_rs::WINDOWS_1258,
            "gbk" | "cp936" | "936" | "gb2312" | "gb18030" => encoding_rs::GBK,
            "euc-kr" | "cp949" | "949" | "uhc" => encoding_rs::EUC_KR,
            "big5" | "cp950" | "950" => encoding_rs::BIG5,
            "ibm866" | "cp866" | "866" => encoding_rs::IBM866,
            _ => {
                if let Some(enc) = Encoding::for_label(name.as_bytes()) {
                    enc
                } else {
                    encoding_rs::WINDOWS_1252
                }
            }
        }
    }

    pub fn encoding_name(&self) -> &str {
        &self.encoding_name
    }

    pub fn decode(&self, bytes: &[u8]) -> String {
        if bytes.is_empty() {
            return String::new();
        }

        // 1. If explicitly configured with a specific encoding (not "auto"), decode with it
        if !self.is_auto && self.encoding != encoding_rs::UTF_8 {
            let (cow, _, had_malformed) = self.encoding.decode(bytes);
            if !had_malformed {
                return cow.into_owned();
            }
        }

        // 2. If it's valid UTF-8, decode directly
        if let Ok(s) = std::str::from_utf8(bytes) {
            return s.to_string();
        }

        // 3. Try Shift-JIS (prevalent in Japanese RM2k/2k3 and RTP defaults like 攻撃)
        let (sjis_cow, _, sjis_malformed) = encoding_rs::SHIFT_JIS.decode(bytes);
        if !sjis_malformed {
            let has_cjk = sjis_cow.chars().any(|c| (c as u32) >= 0x3000);
            if has_cjk {
                return sjis_cow.into_owned();
            }
        }

        // 4. Try GBK / EUC-KR / Big5 if non-ASCII
        let (gbk_cow, _, gbk_malformed) = encoding_rs::GBK.decode(bytes);
        if !gbk_malformed && gbk_cow.chars().any(|c| (c as u32) >= 0x4E00) {
            return gbk_cow.into_owned();
        }

        let (euckr_cow, _, euckr_malformed) = encoding_rs::EUC_KR.decode(bytes);
        if !euckr_malformed && euckr_cow.chars().any(|c| (c as u32) >= 0xAC00) {
            return euckr_cow.into_owned();
        }

        // 5. If Shift-JIS had no malformed errors, use Shift-JIS
        if !sjis_malformed {
            return sjis_cow.into_owned();
        }

        // 6. Default to Windows-1252 for Western European single-byte characters
        let (w1252_cow, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
        w1252_cow.into_owned()
    }

    pub fn encode(&self, text: &str) -> Vec<u8> {
        if text.is_empty() {
            return Vec::new();
        }
        if self.is_auto {
            // When auto-encoding: if it cleanly encodes to Shift-JIS (e.g. CJK text), encode to Shift-JIS
            if text.chars().any(|c| (c as u32) >= 0x3000) {
                let (bytes, _, unmappable) = encoding_rs::SHIFT_JIS.encode(text);
                if !unmappable {
                    return bytes.into_owned();
                }
            }
            // Otherwise try Windows-1252
            let (bytes, _, unmappable) = encoding_rs::WINDOWS_1252.encode(text);
            if !unmappable {
                return bytes.into_owned();
            }
            // Fallback to UTF-8
            return text.as_bytes().to_vec();
        }

        if self.encoding == encoding_rs::UTF_8 {
            return text.as_bytes().to_vec();
        }
        let (bytes, _, _) = self.encoding.encode(text);
        bytes.into_owned()
    }
}
