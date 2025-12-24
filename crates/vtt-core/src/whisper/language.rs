//! Language support

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Language { 
    pub code: &'static str, 
    pub name: &'static str 
}

impl Language {
    pub const fn new(code: &'static str, name: &'static str) -> Self { Self { code, name } }
    
    pub fn from_code(code: &str) -> Option<Self> { 
        SUPPORTED_LANGUAGES.iter().find(|l| l.code == code).copied() 
    }
    
    pub fn is_valid(code: &str) -> bool { 
        SUPPORTED_LANGUAGES.iter().any(|l| l.code == code) || code == "auto" 
    }
}

pub const SUPPORTED_LANGUAGES: &[Language] = &[
    Language::new("en", "English"), 
    Language::new("es", "Spanish"),
    Language::new("fr", "French"), 
    Language::new("de", "German"),
    Language::new("it", "Italian"), 
    Language::new("pt", "Portuguese"),
    Language::new("zh", "Chinese"), 
    Language::new("ja", "Japanese"),
    Language::new("ko", "Korean"), 
    Language::new("ru", "Russian"),
    Language::new("ar", "Arabic"), 
    Language::new("hi", "Hindi"),
];

pub const AUTO_DETECT: &str = "auto";

pub fn supported_codes() -> Vec<&'static str> { 
    SUPPORTED_LANGUAGES.iter().map(|l| l.code).collect() 
}

pub fn display_name(code: &str) -> String {
    if code == AUTO_DETECT { 
        return "Auto-detect".to_string(); 
    }
    Language::from_code(code)
        .map(|l| l.name.to_string())
        .unwrap_or_else(|| format!("Unknown ({})", code))
}
