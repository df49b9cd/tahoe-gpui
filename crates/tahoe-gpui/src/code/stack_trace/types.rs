//! Data types for stack trace parsing.

/// A parsed stack frame.
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_name: Option<String>,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub column_number: Option<u32>,
    pub is_internal: bool,
    pub raw: String,
}

/// A parsed stack trace.
#[derive(Debug, Clone)]
pub struct ParsedStackTrace {
    pub error_type: Option<String>,
    pub error_message: String,
    pub frames: Vec<StackFrame>,
}

/// Parse a stack trace string into structured data.
pub fn parse_stack_trace(trace: &str) -> ParsedStackTrace {
    let lines: Vec<&str> = trace.lines().filter(|l| !l.trim().is_empty()).collect();

    if lines.is_empty() {
        return ParsedStackTrace {
            error_type: None,
            error_message: trace.to_string(),
            frames: Vec::new(),
        };
    }

    let first_line = lines[0].trim();
    let (error_type, error_message) = if let Some(colon_pos) = first_line.find(": ") {
        let prefix = &first_line[..colon_pos];
        if prefix.ends_with("Error") || prefix == "Error" {
            (
                Some(prefix.to_string()),
                first_line[colon_pos + 2..].to_string(),
            )
        } else {
            (None, first_line.to_string())
        }
    } else {
        (None, first_line.to_string())
    };

    let frames: Vec<StackFrame> = lines[1..]
        .iter()
        .filter(|l| l.trim().starts_with("at "))
        .map(|l| parse_frame(l.trim()))
        .collect();

    ParsedStackTrace {
        error_type,
        error_message,
        frames,
    }
}

fn parse_frame(line: &str) -> StackFrame {
    let trimmed = line.trim_start_matches("at ");

    // Pattern: functionName (filePath:line:column)
    if let Some(paren_start) = trimmed.rfind('(')
        && trimmed.ends_with(')')
    {
        let func = trimmed[..paren_start].trim().to_string();
        let location = &trimmed[paren_start + 1..trimmed.len() - 1];
        let (file, line_num, col_num) = parse_location(location);
        let is_internal = file.as_ref().is_some_and(|f| {
            f.contains("node_modules") || f.starts_with("node:") || f.contains("internal/")
        });
        return StackFrame {
            function_name: if func.is_empty() { None } else { Some(func) },
            file_path: file,
            line_number: line_num,
            column_number: col_num,
            is_internal,
            raw: line.to_string(),
        };
    }

    // Pattern: filePath:line:column (no function)
    let (file, line_num, col_num) = parse_location(trimmed);
    let is_internal = file.as_ref().is_some_and(|f| {
        f.contains("node_modules") || f.starts_with("node:") || f.contains("internal/")
    });

    StackFrame {
        function_name: None,
        file_path: file,
        line_number: line_num,
        column_number: col_num,
        is_internal,
        raw: line.to_string(),
    }
}

fn parse_location(s: &str) -> (Option<String>, Option<u32>, Option<u32>) {
    // Peel off column and line from the right, verifying they're numeric.
    // This correctly handles Windows paths (C:\foo\bar.js:10:5) and
    // URLs (http://example.com/file.js:10:5).
    if let Some(last_colon) = s.rfind(':') {
        let candidate = &s[last_colon + 1..];
        if let Ok(col) = candidate.parse::<u32>() {
            let rest = &s[..last_colon];
            if let Some(second_colon) = rest.rfind(':') {
                let candidate2 = &rest[second_colon + 1..];
                if let Ok(line) = candidate2.parse::<u32>() {
                    let file = &rest[..second_colon];
                    if !file.is_empty() {
                        return (Some(file.to_string()), Some(line), Some(col));
                    }
                }
            }
            // Only one numeric suffix — treat as line number
            let file = &s[..last_colon];
            if !file.is_empty() {
                return (Some(file.to_string()), Some(col), None);
            }
        }
    }
    (Some(s.to_string()), None, None)
}
