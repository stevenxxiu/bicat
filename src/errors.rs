use std::io;

use custom_error::custom_error;

custom_error! {pub ProgramError
    ImageError {details: String} = "Image error: {details}",
    Internal {details: String} = "Internal error: {details}", // should not happen
    Io {source: io::Error} = "IO Error : {source}",
    Svg {source: SvgError} = "SVG error: {source}",
}

custom_error! {pub SvgError
    Io {source: io::Error} = "IO Error : {source}",
    Internal { message: &'static str } = "Internal error : {message}",
    Svg {source: resvg::usvg::Error} = "SVG Error: {source}",
}
