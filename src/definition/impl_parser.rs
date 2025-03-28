use super::super::utility::dummy;
use super::{JParser, JResult, JValue, Json, VKind};
use std::collections::HashMap;
impl<'a> JParser<'a> {
  fn next(&mut self) -> Result<char, String> {
    let ch = self.input_code[self.pos..]
      .chars()
      .next()
      .ok_or("Reached end of text")?;
    self.pos += ch.len_utf8();
    Ok(ch)
  }
  fn expect(&mut self, expected: char) -> JResult {
    if self.input_code[self.pos..].starts_with(expected) {
      self.next()?;
      dummy()
    } else {
      self.parse_err(&format!("Expected character '{expected}' not found."))
    }
  }
  pub fn parse(&mut self, code: &'a str) -> JResult {
    self.input_code = code;
    self.pos = 0;
    self.ln = 1;
    let result = self.parse_value()?;
    self.skip_ws();
    if self.pos != self.input_code.len() {
      self.parse_err("Unexpected trailing characters")
    } else {
      Ok(result)
    }
  }
  fn skip_ws(&mut self) {
    while let Some(c) = self.input_code[self.pos..].chars().next() {
      if c.is_whitespace() {
        if c == '\n' {
          self.ln += 1;
        }
        self.pos += c.len_utf8();
      } else {
        break;
      }
    }
  }
  fn parse_name(&mut self, n: &str, v: JValue) -> JResult {
    if self.input_code[self.pos..].starts_with(n) {
      let start = self.pos;
      self.pos += n.len();
      Ok(Json {
        pos: start,
        ln: self.ln,
        value: v,
      })
    } else {
      self.parse_err(&format!("Failed to parse '{n}'"))
    }
  }
  fn parse_number(&mut self) -> JResult {
    let start = self.pos;
    let mut num_str = String::new();
    let mut has_decimal = false;
    let mut has_exponent = false;
    if self.input_code[self.pos..].starts_with('-') {
      num_str.push('-');
      self.next()?;
    }
    if self.input_code[self.pos..].starts_with('0') {
      num_str.push('0');
      self.next()?;
      if matches!(self.input_code[self.pos..].chars().next(), Some(c) if c.is_ascii_digit()) {
        return self.parse_err("Leading zeros are not allowed in numbers");
      }
    } else if matches!(self.input_code[self.pos..].chars().next(), Some('1'..='9')) {
      while let Some(ch) = self.input_code[self.pos..].chars().next() {
        if ch.is_ascii_digit() {
          num_str.push(ch);
          self.next()?;
        } else {
          break;
        }
      }
    } else {
      return self.parse_err("Invalid number format");
    }
    if let Some(ch) = self.input_code[self.pos..].chars().next() {
      if ch == '.' {
        has_decimal = true;
        num_str.push(ch);
        self.next()?;
        if !matches!(self.input_code[self.pos..].chars().next(), Some(c) if c.is_ascii_digit()) {
          return self.parse_err("A digit is required after the decimal point");
        }
        while let Some(ch) = self.input_code[self.pos..].chars().next() {
          if ch.is_ascii_digit() {
            num_str.push(ch);
            self.next()?;
          } else {
            break;
          }
        }
      }
    }
    if let Some(ch) = self.input_code[self.pos..].chars().next() {
      if ch == 'e' || ch == 'E' {
        has_exponent = true;
        num_str.push(ch);
        self.next()?;
        if matches!(self.input_code[self.pos..].chars().next(), Some('+' | '-')) {
          num_str.push(self.next()?);
        }
        if !matches!(self.input_code[self.pos..].chars().next(), Some(c) if c.is_ascii_digit()) {
          return self.parse_err("A digit is required in the exponent part");
        }
        while let Some(ch) = self.input_code[self.pos..].chars().next() {
          if ch.is_ascii_digit() {
            num_str.push(ch);
            self.next()?;
          } else {
            break;
          }
        }
      }
    }
    if !has_decimal && !has_exponent {
      num_str.parse::<i64>().map_or_else(
        |_| self.parse_err("Invalid integer value"),
        |int_val| {
          Ok(Json {
            pos: start,
            ln: self.ln,
            value: JValue::Int(VKind::Lit(int_val)),
          })
        },
      )
    } else {
      num_str.parse::<f64>().map_or_else(
        |_| self.parse_err("Invalid numeric value"),
        |float_val| {
          Ok(Json {
            pos: start,
            ln: self.ln,
            value: JValue::Float(VKind::Lit(float_val)),
          })
        },
      )
    }
  }
  fn parse_string(&mut self) -> JResult {
    if !self.input_code[self.pos..].starts_with('\"') {
      return self.parse_err("Missing opening quotation for string");
    }
    let start = self.pos;
    self.pos += 1;
    let mut result = String::new();
    while self.pos < self.input_code.len() {
      let c = self.next()?;
      match c {
        '\"' => {
          return Ok(Json {
            pos: start,
            ln: self.ln,
            value: JValue::String(VKind::Lit(result)),
          });
        }
        '\\' => {
          let escaped = self.next()?;
          match escaped {
            'n' => result.push('\n'),
            't' => result.push('\t'),
            'r' => result.push('\r'),
            'b' => result.push('\x08'),
            'f' => result.push('\x0C'),
            '\\' => result.push('\\'),
            '/' => result.push('/'),
            '"' => result.push('"'),
            'u' => {
              let mut hex = String::new();
              for _ in 0..4 {
                if let Ok(c) = self.next() {
                  if c.is_ascii_hexdigit() {
                    hex.push(c);
                  } else {
                    return self.parse_err("Invalid hex digit");
                  }
                } else {
                  return self.parse_err("Failed read hex");
                }
              }
              let cp =
                u32::from_str_radix(&hex, 16).map_err(|_| String::from("Invalid code point"))?;
              if (0xD800..=0xDFFF).contains(&cp) {
                return self.parse_err("Invalid unicode");
              }
              result.push(std::char::from_u32(cp).ok_or("Invalid unicode")?);
            }
            _ => {
              return self.parse_err("Invalid escape sequence");
            }
          }
        }
        c if c < '\u{20}' => {
          return self.parse_err("Invalid control character");
        }
        _ => result.push(c),
      }
    }
    self.parse_err("String is not properly terminated")
  }
  fn parse_array(&mut self) -> JResult {
    let start_pos = self.pos;
    let start_ln = self.ln;
    let mut array = Vec::new();
    self.expect('[')?;
    self.skip_ws();
    if self.input_code[self.pos..].starts_with(']') {
      self.pos += 1;
      return Ok(Json {
        pos: start_pos,
        ln: start_ln,
        value: JValue::Array(VKind::Lit(array)),
      });
    }
    loop {
      array.push(self.parse_value()?);
      if self.input_code[self.pos..].starts_with(']') {
        self.pos += 1;
        return Ok(Json {
          pos: start_pos,
          ln: start_ln,
          value: JValue::Array(VKind::Lit(array)),
        });
      } else if self.input_code[self.pos..].starts_with(',') {
        self.pos += 1;
      } else {
        return self.parse_err("Invalid array separator");
      }
    }
  }
  fn parse_object(&mut self) -> JResult {
    let start_pos = self.pos;
    let start_ln = self.ln;
    let mut object = HashMap::new();
    self.expect('{')?;
    self.skip_ws();
    if self.input_code[self.pos..].starts_with('}') {
      self.pos += 1;
      return Ok(Json {
        pos: start_pos,
        ln: start_ln,
        value: JValue::Object(VKind::Lit(object)),
      });
    }
    loop {
      let key = self.parse_value()?;
      let JValue::String(VKind::Lit(s)) = key.value else {
        return self.obj_err("Keys must be strings", &key);
      };
      self.expect(':')?;
      let value = self.parse_value()?;
      object.insert(s, value);
      if self.input_code[self.pos..].starts_with('}') {
        self.pos += 1;
        return Ok(Json {
          pos: start_pos,
          ln: start_ln,
          value: JValue::Object(VKind::Lit(object)),
        });
      }
      if self.input_code[self.pos..].starts_with(',') {
        self.pos += 1
      } else {
        return self.parse_err("Invalid object separator");
      }
    }
  }
  fn parse_value(&mut self) -> JResult {
    self.skip_ws();
    if self.pos >= self.input_code.len() {
      return self.parse_err("Unexpected end of text");
    }
    let result = match self.input_code[self.pos..].chars().next() {
      Some('"') => self.parse_string(),
      Some('{') => self.parse_object(),
      Some('[') => self.parse_array(),
      Some('t') => self.parse_name("true", JValue::Bool(VKind::Lit(true))),
      Some('f') => self.parse_name("false", JValue::Bool(VKind::Lit(false))),
      Some('n') => self.parse_name("null", JValue::Null),
      _ => self.parse_number(),
    };
    self.skip_ws();
    result
  }
}
