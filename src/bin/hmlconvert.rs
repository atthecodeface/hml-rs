use std::fs::File;
use std::io::{Read, Write};

use clap::{value_parser, Arg, Command};

use hml_rs::hml_reader::Parser;
use hml_rs::names::{Namespace, NamespaceStack};

use lexer_rs::FmtContext;
use lexer_rs::{Lexer, LineColumn, StreamCharPos};

type LexerPos = StreamCharPos<LineColumn>;
type HmlError = hml_rs::HmlError<LexerPos>;

fn main() {
    let matches = Command::new("hml")
        .about("HML parser to output file generator")
        .after_help(
            "This program reads an HML (or in the future XML) file, and outputs an XML file",
        )
        .author("Gavin J Stark")
        .version("0.1")
        .arg(
            Arg::new("output")
                .long("output")
                .help("Sets the output file to use")
                .required(false)
                .num_args(1), // was takes_value(true)
        )
        .arg(
            Arg::new("xml_version")
                .short('x')
                .long("xml_version")
                .help("XML version to use (if HML to XML conversion)")
                .required(false)
                .num_args(1) // was takes_value(true)
                .value_parser(value_parser!(f32)),
        )
        .arg(Arg::new("file").help("Input file to read"))
        .get_matches();

    let mut xml_version = 100;
    if let Some(x) = matches.get_one::<f32>("xml_version") {
        xml_version = (x * 100.0).round() as usize;
    }

    let mut text = String::new();
    match matches.get_one::<String>("file") {
        None => {
            std::io::stdin().read_to_string(&mut text).unwrap();
        }
        Some(filename) => {
            let mut file = File::open(filename).unwrap();
            file.read_to_string(&mut text).unwrap();
        }
    }
    let mut namespace = Namespace::new(false);
    let mut namespace_stack = NamespaceStack::new(&mut namespace);
    let lexer_string = lexer_rs::LexerOfString::default().set_text(text);
    let lexer = lexer_string.lexer();
    let lexer_parsers = hml_rs::hml_reader::parse_fns();
    let mut lexer_iter = lexer.iter(&lexer_parsers);
    let mut parser: Parser<LexerPos> = Parser::default().set_version(xml_version);
    let output = std::io::stdout();
    let mut writer = xml::writer::EmitterConfig::new()
        .perform_indent(true)
        .create_writer(output);
    loop {
        match parser.next_event(&mut namespace_stack, || lexer_iter.next()) {
            Ok(event) => {
                if let Some(x) = event.as_xml_writer(&namespace_stack) {
                    writer.write(x).unwrap();
                } else {
                    break;
                }
            }
            Err(e) => {
                let e: HmlError = e;
                if let Some(span) = e.span() {
                    let mut s = String::new();
                    // reader
                    lexer_string
                        .fmt_context(&mut s, span.start(), span.end())
                        .unwrap();
                    write!(&mut std::io::stderr(), "{}", s).unwrap();
                    eprintln!("Parse error: {e}");
                    eprintln!();
                }
                break;
            }
        }
    }
}
