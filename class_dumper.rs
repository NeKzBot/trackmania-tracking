// rustc -O class_dumper.rs

struct Pattern<'a, T> {
    offsets: (usize, usize, usize),
    signature: &'a [T],
}

struct ClassInfo {
    size: i32,
    name: String,
    id: i32,
}

fn read_i32(buffer: &[u8], offset: usize) -> i32 {
    (buffer[offset] as i32
        + ((buffer[offset + 1] as i32) << 8i32)
        + ((buffer[offset + 2] as i32) << 16i32)
        + ((buffer[offset + 3] as i32) << 24i32)) as i32
}

fn read_cstr(buffer: &[u8], offset: usize) -> String {
    let mut result = String::new();
    let mut offset = offset;

    while buffer[offset] != 0x00 {
        result.push(buffer[offset] as char);
        offset += 1;
    }

    result
}

fn deref<T, V>(r#type: &V, buffer: &[u8], offset: usize) -> T
where
    T: Sized,
    V: Fn(&[u8], usize) -> T,
{
    r#type(
        buffer,
        offset + read_i32(buffer, offset) as usize + std::mem::size_of::<i32>(),
    )
}

macro_rules! pattern {
    ($bytes:expr, $offsets:expr) => {
        Pattern {
            offsets: $offsets,
            signature: &$bytes,
        };
    };
}

fn main() -> std::io::Result<()> {
    let args = std::env::args().collect::<Vec<String>>();

    let game = &args.get(1).expect("game name required as first argument");
    let exe = &args.get(2).expect("exe path required as second argument");

    let buffer: Vec<u8> = std::fs::read(exe)?;
    let buffer_length = buffer.len();

    #[rustfmt::skip]
    let patterns = vec!(
        pattern!([
                0x48, 0x83, 0xEC, 0x38,
                0x48, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00,
                0xC7, 0x44, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, // mov [rsp+38h+var_10], m_iSize
                0x4C, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,       // lea r9, m_szClassName
                0x48, 0x89, 0x44, 0x24, 0x00,
                0x4C, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00,
                0xBA,
            ],
            (15, 22, 39)
        ),
        pattern!([
                0x48, 0x83, 0xEC, 0x38,
                0xC7, 0x44, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00,       // mov [rsp+38h+var_10], m_iSize
                0x4C, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,             // lea r9, m_szClassName
                0x4C, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00,
                0x48, 0xC7, 0x44, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00,
                0xBA, 0x00, 0x00, 0x00, 0x00,
                0x48, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,             //  mov edx, m_iClassId
                0xE8, 0x00, 0x00, 0x00, 0x00,
                0x48, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,
                0x48, 0x89, 0x0D, 0x00, 0x00, 0x00, 0x00,
                0x48, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,
                0x48, 0x83, 0xC4, 0x38,
                0xE9,
            ],
            (8, 15, 43)
        ),
        pattern!([
                0x48, 0x83, 0xEC, 0x38,
                0xC7, 0x44, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00,       // mov [rsp+38h+var_10], m_iSize
                0x4C, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,             // lea r9, m_szClassName
                0x4C, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00,
                0x48, 0xC7, 0x44, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00,
                0xBA, 0x00, 0x00, 0x00, 0x00,                         //  mov edx, m_iClassId
                0x48, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,
                0xE8, 0x00, 0x00, 0x00, 0x00,
                0x48, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,
                0x48, 0x83, 0xC4, 0x38,
                0xE9,
            ],
            (8, 15, 36)
        ),
    );

    let mut classes: Vec<ClassInfo> = Vec::new();

    for pattern in patterns {
        let signature_length = pattern.signature.len();

        for byte_index in 0..buffer_length {
            let mut matches = 0;

            for (sig_index, byte) in pattern.signature.iter().enumerate() {
                if *byte != 0x00 && *byte != buffer[byte_index + sig_index] {
                    break;
                }

                matches += 1;
            }

            if matches == signature_length {
                let size = read_i32(buffer.as_slice(), byte_index + pattern.offsets.0);
                let name = deref(
                    &read_cstr,
                    buffer.as_slice(),
                    byte_index + pattern.offsets.1,
                );
                let id = read_i32(buffer.as_slice(), byte_index + pattern.offsets.2);

                classes.push(ClassInfo {
                    size,
                    name: name.clone(),
                    id,
                });
            }
        }
    }

    classes.sort_by(|a: &ClassInfo, b: &ClassInfo| a.name.cmp(&b.name));

    println!("# {} Classes\r\n", game);
    println!(
        "Dumped game classes from {}.\r\n",
        exe.chars()
            .skip(
                exe.rfind('/')
                    .unwrap_or_else(|| exe.rfind('\\').unwrap_or(0))
                    + 1
            )
            .collect::<String>()
    );
    println!("|Class|Id|Size|\r\n|---|:-:|:-:|");

    for info in classes {
        println!("|{}|0x{:x}|{}|", info.name, info.id, info.size);
    }

    Ok(())
}
