#![feature(path_file_prefix)]

use regex::Regex;
use std::{
    error::Error,
    fs::OpenOptions,
    io::Write,
    fmt::Display,
};
#[derive(Debug, Clone)]
pub struct UnparsableFileError {
    inner : String
}

impl Error for UnparsableFileError{}
impl Display for UnparsableFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cannot parse file, because {}", self.inner)
    }

    
}
impl From<csv::Error> for UnparsableFileError {
    fn from(value: csv::Error) -> Self {
        UnparsableFileError { inner: value.to_string() }
    }
}

/// Takes in a Edinburgh Instrument LFP file and returns a glotaran compatible
/// .ascii file in the `wavelength explicit` format
/// ```
/// use glotaran_converter_lib::run_lfp;
/// let filename : &str = "example_lfp.txt"; // Valid EI file
/// let new_filename = run_lfp(filename).expect("Error converting/reading file");
/// let prefix_a = format!("{}",filename.split_once(".").unwrap().0);
/// let prefix_b = format!("{}",new_filename.split_once(".").unwrap().0);
/// assert_eq!(prefix_a, prefix_b);
/// ```
pub fn run_lfp(source : &str) -> Result<String, Box<dyn std::error::Error>> {
    let re = Regex::new(r"(\d){3}").unwrap();
    let mut rdr = csv::ReaderBuilder::new().delimiter(b',').flexible(true).from_path(source)?;
    let headers_raw = rdr.records().next().unwrap()?;
    let headers = headers_raw.into_iter().map(|col| {
        match re.find(col) {
            Some(mtch) => mtch.as_str(),
            None => ""
        }
    }).collect::<Vec<_>>();
    let mut body :Vec<Vec<String>> = vec![];
    for record in rdr.records().skip(8) {
        let record_vec = record?.into_iter().map(|s|s.to_owned()).collect();
        body.push(record_vec)
    }
    let output_filename = format!("{}.ascii",source.split_once(".").unwrap_or(("file","")).0);
    let headlines = headers.len() - 1; // -1 porque se agrega una columna vacia donde están los tiempos
    let filename = write_to_file(headers, body, headlines, &output_filename).unwrap();
    Ok(filename)
    }
    

/// Takes in a Horiba DataStation text file (generated in datastation software, copying all traces to clipboard) and returns a glotaran compatible
/// .ascii file in the `wavelength explicit` format
/// ```
/// use glotaran_converter_lib::run_das6;
/// let filename : &str = "example_trp.txt"; // Valid DataStation file
/// let output_filename : String = "example_trp.ascii".to_owned();
/// let sync_delay : f32 = 0f32;
/// let ns_per_chn : f32 = 2.5e4;
/// let new_filename = run_das6(filename, sync_delay, ns_per_chn,output_filename).expect("Error converting/reading file");
/// let prefix_a = format!("{}",filename.split_once(".").unwrap().0);
/// let prefix_b = format!("{}",new_filename.split_once(".").unwrap().0);
/// assert_eq!(prefix_a, prefix_b);
/// ```
pub fn run_das6(source : &str, sync_delay : f32, ns_per_chn : f32, output_filename : String) -> Result<String, UnparsableFileError> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(source)
        .expect("Problema leyendo archivo");
    let re = Regex::new(r"(\d){3}").unwrap();
    let mut body: Vec<Vec<_>> = vec![];
    for (n, record) in rdr.records().enumerate() {
        let mut line = record?
            .into_iter()
            .map(|recn| recn.to_string())
            .collect::<Vec<String>>();
        line.insert(0, format!("{}", (( n as f32 - sync_delay )* ns_per_chn ) as i32));
        line.remove(1); // Drop prompt
        body.push(line.clone()); // desclonar later
    }
    let mut headers = rdr
        .headers()?
        .into_iter()
        .filter_map(|rec| {
            match re.captures(rec) {
                None => None,
                Some(caps) => caps.get(0).map_or(Some("0"), |m| Some(m.as_str()) )
            }
            })
        .collect::<Vec<&str>>();
    headers.push("");
    let headlines = headers.len();
    let filename = write_to_file(headers, body, headlines, &output_filename).unwrap();
    Ok(filename)
}

fn write_to_file(
    headers: Vec<&str>,
    body: Vec<Vec<String>>,
    line_number: usize,
    output_filename : &str
) -> Result<String, Box<dyn Error>> {
    let filename = output_filename;
    let mut file = OpenOptions::new()
    .append(true)
    .write(true)
    .create(true)
    .open(&filename)?;
    writeln!(file, "{filename}")?;
    writeln!(file, "Eduardo Gonik")?;
    writeln!(file, "wavelength explicit")?;
    // file.write(format!("interval nr {}", (line_number - 1)).as_bytes())?;
    let ii = format!("intervalnr {}", (line_number - 1));
    writeln!(file, "{ii}")?;
    file.flush()?;
    let file2 = OpenOptions::new().append(true).open(&filename)?;
    let mut writer = csv::WriterBuilder::new()
    .delimiter(b'\t')
    .from_writer(file2);
    // let mut writer = csv::Writer::from_writer(file2);
    writer.write_record(&headers).unwrap();
    body.into_iter()
        .for_each(|v| writer.write_record(&v).unwrap());
    writer.flush()?;
    Ok(filename.into())
}
