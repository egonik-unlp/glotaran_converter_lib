use anyhow::Context;
use itertools::Itertools;
use regex::Regex;
use std::{error::Error, fmt::Display, fs::OpenOptions, io::Write, ops::Index};
#[derive(Debug, Clone)]
pub struct UnparsableFileError {
    inner: String,
}

impl Error for UnparsableFileError {}
impl Display for UnparsableFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cannot parse file, because {}", self.inner)
    }
}
impl From<csv::Error> for UnparsableFileError {
    fn from(value: csv::Error) -> Self {
        UnparsableFileError {
            inner: value.to_string(),
        }
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
pub fn run_lfp(source: &str) -> anyhow::Result<String> {
    let re = Regex::new(r"(\d){3}").unwrap();
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b',')
        .flexible(true)
        .from_path(source)
        .context("Source file could not be read")?;
    let headers_raw = rdr.records().next().unwrap()?;
    let headers = headers_raw
        .into_iter()
        .map(|col| match re.find(col) {
            Some(mtch) => mtch.as_str(),
            None => "",
        })
        .collect::<Vec<_>>();
    let mut body: Vec<Vec<String>> = vec![];
    for record in rdr.records().skip(8) {
        let record_vec = record?.into_iter().map(|s| s.to_owned()).collect();
        body.push(record_vec)
    }
    let output_filename = format!("{}.ascii", source.split_once(".").unwrap_or(("file", "")).0);
    let headlines = headers.len() - 1; // -1 porque se agrega una columna vacia donde están los tiempos
    let filename = write_to_file(headers, body, headlines, &output_filename)
        .context("Output file couldn't be written")?;
    anyhow::Ok(filename)
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
pub fn run_das6(
    source: &str,
    sync_delay: f32,
    ns_per_chn: f32,
    output_filename: String,
) -> Result<String, UnparsableFileError> {
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
        line.insert(
            0,
            format!("{}", ((n as f32 - sync_delay) * ns_per_chn) as i32),
        );
        line.remove(1); // Drop prompt
        body.push(line.clone()); // desclonar later
    }
    let mut headers = rdr
        .headers()?
        .into_iter()
        .filter_map(|rec| match re.captures(rec) {
            None => None,
            Some(caps) => caps.get(0).map_or(Some("0"), |m| Some(m.as_str())),
        })
        .collect::<Vec<&str>>();
    headers.push("");
    let headlines = headers.len();
    let filename = write_to_file(headers, body, headlines, &output_filename).unwrap();
    Ok(filename)
}
pub fn run_r4(filename: String) -> anyhow::Result<String> {
    let output_filename = {
        let stub = filename.split_once(".").unwrap().0;
        format!("{}.ascii", stub)
    };
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(filename)
        .context("Couldn't open lfp file")?;
    let mut document: Vec<_> = reader
        .headers()
        .context("Couldn't read headers")?
        .iter()
        .map(|record| vec![record.to_owned()])
        .collect();
    reader.into_records().for_each(|record| {
        for (index, cell) in record.unwrap().iter().enumerate() {
            document.get_mut(index).unwrap().push(cell.to_owned());
        }
    });
    document.sort_by_key(|row| {
        let key = row.first().unwrap();
        key.parse::<i32>().expect("Couldn't parse wavelength")
    });
    let mut return_headers = vec![];
    let col_length = return_headers.len();
    let mut return_body = vec![];
    for (col_num, column) in document.iter().enumerate() {
        for (row_n, cell) in column.into_iter().enumerate() {
            if row_n == 0 {
                return_headers.push(cell.as_str());
            } else if col_num == 0 {
                return_body.push(vec![cell.to_owned()]);
            }
            return_body
                .get_mut(row_n - 1)
                .unwrap()
                .push(cell.to_owned());
        }
    }
    write_to_file(
        return_headers,
        return_body,
        col_length,
        output_filename.as_str(),
    )
    .context("Couldn't write to file")?;
    return anyhow::Ok(output_filename);
}

fn write_to_file(
    headers: Vec<&str>,
    body: Vec<Vec<String>>,
    line_number: usize,
    output_filename: &str,
) -> anyhow::Result<String> {
    let filename = output_filename;
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)?;
    writeln!(file, "{filename}")?;
    writeln!(file, "Eduardo Gonik")?;
    writeln!(file, "wavelength explicit")?;
    // file.write(format!("interval nr {}", (line_number - 1)).as_bytes())?;
    let ii = format!("intervalnr {}", (line_number - 1));
    writeln!(file, "{ii}")?;
    file.flush()?;
    let file2 = OpenOptions::new().append(true).open(filename)?;
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(file2);
    // let mut writer = csv::Writer::from_writer(file2);
    writer.write_record(&headers).unwrap();
    body.into_iter()
        .for_each(|v| writer.write_record(&v).unwrap());
    writer.flush()?;
    anyhow::Ok(filename.into())
}
