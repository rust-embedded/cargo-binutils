extern crate cargo_binutils as cbu;

use std::borrow::Cow;
use std::process;

fn main() {
    match cbu::run(
        false,
        |ctxt| ctxt.size(),
        |ctxt, stdout| {
            if ctxt.tool_args().iter().any(|arg| arg == "-A")
                && !ctxt.tool_args().iter().any(|arg| arg == "-x")
            {
                stdout
                    .lines()
                    .map(|line| -> Cow<str> {
                        let mut parts = line.split_whitespace();

                        if let Some((needle, addr)) = parts
                            .nth(2)
                            .and_then(|part| part.parse::<u64>().ok().map(|addr| (part, addr)))
                        {
                            let pos = line.rfind(needle).unwrap();
                            let hex_addr = format!("{:#x}", addr);
                            let start = pos + needle.as_bytes().len() - hex_addr.as_bytes().len();

                            format!("{}{}", &line[..start], hex_addr).into()
                        } else {
                            line.into()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
                    .into()
            } else {
                stdout.into()
            }
        },
    ) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
