use std::collections::HashMap;

use super::{utility::GetNumFromBytes, *};
use crate::maker::*;

const TIFF_LITTLE_ENDIAN: u16 = 0x4949;
const TIFF_BIG_ENDIAN: u16 = 0x4d4d;

impl Value {
    fn size(&self) -> usize {
        match self {
            Value::U16(_) => 2,
            Value::U32(_) => 4,
            Value::Str(_) => 1,
            Value::R64(_) => 8,
        }
    }
}

impl<'a> Parser<'a> {
    fn is_byte_order_le(buffer: &[u8], start: usize) -> Result<bool, ParsingError> {
        let byte_order = buffer.u16(true, start);
        match byte_order {
            TIFF_LITTLE_ENDIAN => Ok(true),
            TIFF_BIG_ENDIAN => Ok(false),
            _ => Err(ParsingError::InvalidTiffHeaderByteOrder(byte_order)),
        }
    }

    pub fn get_raw_info(buffer: &[u8], task: &ExifTask) -> Result<ParsedRawInfo, ParsingError> {
        let content = HashMap::new();
        Self::get_raw_info_with_content(buffer, task, content)
    }
    pub fn get_raw_info_with_content(
        buffer: &[u8],
        task: &ExifTask,
        mut content: HashMap<String, Value>,
    ) -> Result<ParsedRawInfo, ParsingError> {
        let buffer = if &buffer[..2] == [0xff, 0xd8] {
            // JPEG header fix
            &buffer[12..]
        } else {
            &buffer
        };

        let decoder = Parser {
            is_le: Parser::is_byte_order_le(buffer, 0)?,
            buffer,
            offset: 0,
            entires: HashMap::new(),
            next_offset: 0,
        };
        decoder.run_task_body(task, &mut content)?;
        Ok(ParsedRawInfo {
            is_le: decoder.is_le,
            content,
        })
    }

    fn is_tasks_in_ifd(tasks: &[ExifTask]) -> bool {
        tasks.iter().any(|x| {
            matches!(
                x,
                ExifTask::Jump {
                    tag: _,
                    is_optional: _,
                    tasks: _
                } | ExifTask::TagItem {
                    tag: _,
                    name: _,
                    len: _,
                    is_optional: _,
                    is_value_u16: _,
                } | ExifTask::SonyDecrypt {
                    offset_tag: _,
                    len_tag: _,
                    key_tag: _,
                    tasks: _
                }
            )
        })
    }
    fn read_u32value_from_entries(&self, tag: u16, custom_offset: Option<usize>) -> Result<u32, ParsingError> {
        let tag_line = self.entires.get(&tag).ok_or(ParsingError::TagNotFound(tag))?;
        Ok(tag_line.u32(self.is_le, custom_offset.unwrap_or(8)))
    }
    fn read_u16value_from_entries(&self, tag: u16, custom_offset: Option<usize>) -> Result<u16, ParsingError> {
        let tag_line = self.entires.get(&tag).ok_or(ParsingError::TagNotFound(tag))?;
        Ok(tag_line.u16(self.is_le, custom_offset.unwrap_or(8)))
    }
    fn read_value_from_offset(&self, offset: usize, t: &Value) -> Value {
        let offset = self.offset + offset * t.size();
        match t {
            Value::U16(_) => Value::U16(self.buffer.u16(self.is_le, offset)),
            Value::U32(_) => Value::U32(self.buffer.u32(self.is_le, offset)),
            Value::Str(_) => {
                let str: String = self.buffer[offset..]
                    .iter()
                    .map_while(|&x| if x == 0 { None } else { Some(x as char) })
                    .collect();
                Value::Str(str.trim().to_owned())
            }
            Value::R64(_) => Value::R64(self.buffer.r64(self.is_le, offset)),
        }
    }

    fn run_remain_tasks(&mut self, tasks: &[ExifTask], content: &mut HashMap<String, Value>) -> Result<(), ParsingError> {
        // IFD entry check
        if Parser::is_tasks_in_ifd(tasks) {
            let entry_count = self.buffer.u16(self.is_le, self.offset) as usize;
            self.offset += 2;

            for tag_line in self.buffer[self.offset..].chunks_exact(12).take(entry_count) {
                let tag = tag_line.u16(self.is_le, 0);
                self.entires.insert(tag, tag_line);
            }

            self.next_offset = self.buffer.u32(self.is_le, self.offset + entry_count * 12) as usize;
        }

        for task in tasks.iter() {
            self.run_task_body(task, content)?;
        }
        Ok(())
    }
    fn run_task_body(&self, task: &ExifTask, content: &mut HashMap<String, Value>) -> Result<(), ParsingError> {
        match task {
            // blocks
            ExifTask::Tiff(tasks) => {
                let is_le = if self.offset == 0 {
                    self.is_le
                } else {
                    Parser::is_byte_order_le(self.buffer, self.offset)?
                };

                let new_buffer = &self.buffer[self.offset..];
                let mut new_parser = Parser {
                    is_le,
                    buffer: new_buffer,
                    offset: new_buffer.u32(is_le, 4) as usize,
                    entires: HashMap::new(),
                    next_offset: 0,
                };
                new_parser.run_remain_tasks(tasks, content)?;
            }
            &ExifTask::Condition {
                cond,
                ref left,
                ref right,
            } => {
                let (cond_type, field, target) = cond;
                let result = match cond_type {
                    CondType::LT | CondType::EQ | CondType::GT => {
                        let value = content
                            .get(field)
                            .ok_or_else(|| RawInfoError::FieldNotFound(field.to_owned()))?
                            .u32().map_err(|err| RawInfoError::ValueError(err))?;

                        match cond_type {
                            CondType::LT => value < target,
                            CondType::EQ => value == target,
                            CondType::GT => value > target,
                            _ => value == target,
                        }
                    }
                    CondType::EXIST => content.get(field).is_some(),
                };

                for task in if result { left } else { right }.iter() {
                    self.run_task_body(task, content)?;
                }
            }
            &ExifTask::JumpNext(ref tasks) => {
                let mut new_parser = Parser {
                    is_le: self.is_le,
                    buffer: self.buffer,
                    offset: self.next_offset,
                    entires: HashMap::new(),
                    next_offset: 0,
                };
                new_parser.run_remain_tasks(tasks, content)?;
            }
            &ExifTask::Jump {
                tag,
                is_optional,
                ref tasks,
            } => {
                let offset = self.read_u32value_from_entries(tag, None);
                match (offset, is_optional) {
                    (Ok(offset), _) => {
                        let mut new_parser = Parser {
                            is_le: self.is_le,
                            buffer: self.buffer,
                            offset: offset as usize,
                            entires: HashMap::new(),
                            next_offset: 0,
                        };
                        new_parser.run_remain_tasks(tasks, content)?;
                    }
                    (Err(e), false) => Err(e)?,
                    _ => {}
                }
            }
            &ExifTask::Scan {
                marker,
                name,
                ref tasks,
            } => {
                let &(offset, _) = &self.buffer[self.offset..]
                    .windows(marker.len())
                    .enumerate()
                    .find(|(_, data)| data == &marker)
                    .ok_or_else(|| ParsingError::ScanFailed(marker))?;

                let tiff_offset = offset + self.offset;
                if let Some(n) = name {
                    content.insert(n.to_owned(), Value::U32(tiff_offset as u32));
                }

                let mut new_parser = Parser {
                    is_le: self.is_le,
                    buffer: &self.buffer[tiff_offset..],
                    offset: 0,
                    entires: HashMap::new(),
                    next_offset: 0,
                };
                new_parser.run_remain_tasks(tasks, content)?;
            }
            &ExifTask::Offset(ref offset, ref tasks) => {
                let new_offset = match offset {
                    OffsetType::Bytes(0) => {
                        for task in tasks.iter() {
                            self.run_task_body(task, content)?;
                        }
                        return Ok(());
                    }
                    OffsetType::Bytes(x) => (self.offset as isize + x) as usize,
                    OffsetType::Address => (&self.buffer[self.offset..]).u32(self.is_le, 0) as usize,
                    &OffsetType::PrevField(field) => {
                        self.offset
                            + content
                                .get(field)
                                .ok_or_else(|| RawInfoError::FieldNotFound(field.to_owned()))?
                                .usize().map_err(|err| RawInfoError::ValueError(err))?
                    }
                };
                let mut new_parser = Parser {
                    is_le: self.is_le,
                    buffer: self.buffer,
                    offset: new_offset,
                    entires: HashMap::new(),
                    next_offset: 0,
                };
                new_parser.run_remain_tasks(tasks, content)?;
            }
            &ExifTask::SonyDecrypt {
                offset_tag,
                len_tag,
                key_tag,
                ref tasks,
            } => {
                let offset = self.read_u32value_from_entries(offset_tag, None)? as usize;
                let len = self.read_u32value_from_entries(len_tag, None)? as usize;
                let key = self.read_u32value_from_entries(key_tag, None)?;
                let mut decrypted = vec![0u8; offset];
                decrypted.append(&mut sony::utility::sony_decrypt(
                    &self.buffer[offset..offset + len],
                    key,
                    self.is_le,
                ));

                let mut new_parser = Parser {
                    is_le: self.is_le,
                    buffer: &decrypted,
                    offset,
                    entires: HashMap::new(),
                    next_offset: 0,
                };
                new_parser.run_remain_tasks(tasks, content)?;
            }
            // items
            &ExifTask::TagItem {
                tag,
                name,
                len,
                is_optional,
                is_value_u16,
            } => {
                let value = if is_value_u16 {
                    self.read_u16value_from_entries(tag, None)
                        .and_then(|x| Ok(Value::U16(x)))
                } else {
                    self.read_u32value_from_entries(tag, None)
                        .and_then(|x| Ok(Value::U32(x)))
                };
                match (value, is_optional) {
                    (Ok(v), _) => {
                        content.insert(name.to_owned(), v);
                        if let Some(len_name) = len {
                            let value = self.read_u32value_from_entries(tag, Some(4))?;
                            content.insert(len_name.to_owned(), Value::U32(value));
                        }
                    }
                    (Err(e), false) => Err(e)?,
                    _ => {}
                };
            }
            &ExifTask::OffsetItem { offset, name, ref t } => {
                let value = self.read_value_from_offset(offset, t);
                content.insert(name.to_owned(), value);
            }
        };
        Ok(())
    }
}
