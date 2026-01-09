use quick_xml::events::{Event, BytesEnd, BytesStart};
use quick_xml::writer::Writer;
use std::io::Cursor;
use serde::Serialize;

pub const SUBSONIC_API_VERSION: &str = "1.16.1";

#[derive(Debug)]
pub struct SubsonicResponse {
    status: String,
    version: String,
    data: Option<String>,
    error: Option<SubsonicError>,
}

#[derive(Debug, Serialize)]
pub struct SubsonicError {
    code: i32,
    message: String,
}

impl SubsonicResponse {
    pub fn ok() -> Self {
        Self {
            status: "ok".to_string(),
            version: SUBSONIC_API_VERSION.to_string(),
            data: None,
            error: None,
        }
    }

    pub fn error(code: i32, message: String) -> Self {
        Self {
            status: "failed".to_string(),
            version: SUBSONIC_API_VERSION.to_string(),
            data: None,
            error: Some(SubsonicError { code, message }),
        }
    }

    pub fn with_data<T: Serialize>(data: &T) -> Result<Self, String> {
        let xml = quick_xml::se::to_string_with_root("data", data)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            status: "ok".to_string(),
            version: SUBSONIC_API_VERSION.to_string(),
            data: Some(xml),
            error: None,
        })
    }

    pub fn to_xml(&self) -> String {
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        let mut response_attrs = Vec::new();
        response_attrs.push(("status", self.status.as_str()));
        response_attrs.push(("version", self.version.as_str()));

        let start = BytesStart::new("subsonic-response", response_attrs);
        writer.write_event(Event::Start(start)).unwrap();

        if let Some(ref error) = self.error {
            let mut error_attrs = Vec::new();
            error_attrs.push(("code", error.code.to_string().as_str()));
            error_attrs.push(("message", error.message.as_str()));
            let error_elem = BytesStart::new("error", error_attrs);
            writer.write_event(Event::Empty(error_elem)).unwrap();
        }

        if let Some(ref data) = self.data {
            let data_elem = BytesStart::new("data", Vec::new());
            writer.write_event(Event::Start(data_elem)).unwrap();
            writer.write_event(Event::Text(quick_xml::events::Text::new(data))).unwrap();
            writer.write_event(Event::End(BytesEnd::new("data"))).unwrap();
        }

        writer.write_event(Event::End(BytesEnd::new("subsonic-response"))).unwrap();

        let result = writer.into_inner().into_inner();
        String::from_utf8(result).unwrap()
    }
}

impl<T: Serialize> From<T> for SubsonicResponse {
    fn from(data: T) -> Self {
        Self::with_data(&data).unwrap_or_else(|_| {
            Self::error(0, "Failed to serialize response".to_string())
        })
    }
}

pub fn to_subsonic_xml<T: Serialize>(data: &T, root_element: &str) -> Result<String, String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let mut root_attrs = Vec::new();
    root_attrs.push(("status", "ok"));
    root_attrs.push(("version", SUBSONIC_API_VERSION));

    let start = BytesStart::new("subsonic-response", root_attrs);
    writer.write_event(Event::Start(start)).unwrap();

    let data_attrs = Vec::new();
    let data_start = BytesStart::new(root_element, data_attrs);
    writer.write_event(Event::Start(data_start)).unwrap();

    let data_str = quick_xml::se::to_string(data).map_err(|e| e.to_string())?;
    writer.write_event(Event::Text(quick_xml::events::Text::new(&data_str))).unwrap();

    writer.write_event(Event::End(BytesEnd::new(root_element))).unwrap();
    writer.write_event(Event::End(BytesEnd::new("subsonic-response"))).unwrap();

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| e.to_string())
}

pub fn subsonic_ok() -> String {
    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}"/>"#, SUBSONIC_API_VERSION)
}

pub fn subsonic_error(code: i32, message: &str) -> String {
    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="failed" version="{}">
    <error code="{}" message="{}"/>
</subsonic-response>"#, SUBSONIC_API_VERSION, code, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[test]
    fn test_ok_response() {
        let response = SubsonicResponse::ok();
        let xml = response.to_xml();
        assert!(xml.contains(r#"status="ok""#));
        assert!(xml.contains(r#"version="1.16.1""#));
    }

    #[test]
    fn test_error_response() {
        let response = SubsonicResponse::error(10, "Test error".to_string());
        let xml = response.to_xml();
        assert!(xml.contains(r#"status="failed""#));
        assert!(xml.contains(r#"<error code="10" message="Test error""#));
    }

    #[test]
    fn test_error_response_with_special_chars() {
        let response = SubsonicResponse::error(40, "Error with <>&\" chars".to_string());
        let xml = response.to_xml();
        assert!(xml.contains(r#"status="failed""#));
        assert!(xml.contains(r#"message="Error with""#));
    }

    #[test]
    fn test_response_with_data() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };
        let response = SubsonicResponse::with_data(&data);
        assert!(response.is_ok());
        let xml = response.unwrap().to_xml();
        assert!(xml.contains(r#"status="ok""#));
        assert!(xml.contains(r#"<data>"#));
    }

    #[test]
    fn test_subsonic_ok_function() {
        let xml = subsonic_ok();
        assert!(xml.contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(xml.contains(r#"status="ok""#));
        assert!(xml.contains(r#"version="1.16.1""#));
    }

    #[test]
    fn test_subsonic_error_function() {
        let xml = subsonic_error(70, "Not found");
        assert!(xml.contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(xml.contains(r#"status="failed""#));
        assert!(xml.contains(r#"code="70""#));
        assert!(xml.contains(r#"message="Not found""#));
    }

    #[test]
    fn test_to_subsonic_xml() {
        let data = TestData {
            name: "artist".to_string(),
            value: 1,
        };
        let result = to_subsonic_xml(&data, "testElement");
        assert!(result.is_ok());
        let xml = result.unwrap();
        assert!(xml.contains(r#"status="ok""#));
        assert!(xml.contains(r#"version="1.16.1""#));
        assert!(xml.contains("<testElement>"));
    }

    #[test]
    fn test_response_contains_xml_declaration() {
        let response = SubsonicResponse::ok();
        let xml = response.to_xml();
        assert!(xml.starts_with(r#"<subsonic-response"#));
    }

    #[test]
    fn test_error_response_closes_tag() {
        let response = SubsonicResponse::error(50, "Test".to_string());
        let xml = response.to_xml();
        assert!(xml.ends_with("</subsonic-response>"));
    }
}
