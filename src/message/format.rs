pub enum ContentFormat {
    TextPlain,
    LinkFormat,
    Xml,
    OctetStream,
    Exi,
    Json,
}

impl ContentFormat {
    pub fn parse(id: u16) -> Option<ContentFormat> {
        match id {
            0 => Some(ContentFormat::TextPlain),
            40 => Some(ContentFormat::LinkFormat),
            41 => Some(ContentFormat::Xml),
            42 => Some(ContentFormat::OctetStream),
            47 => Some(ContentFormat::Exi),
            50 => Some(ContentFormat::Json),
            _ => None,
        }
    }

    pub fn id(&self) -> u16 {
        match self {
            ContentFormat::TextPlain => 0,
            ContentFormat::LinkFormat => 40,
            ContentFormat::Xml => 41,
            ContentFormat::OctetStream => 42,
            ContentFormat::Exi => 47,
            ContentFormat::Json => 50,
        }
    }
}
