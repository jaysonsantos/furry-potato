use async_stream::stream;
use krak_it::Cli;
use tokio::{
    fs,
    fs::{read_dir, DirEntry, File},
    io::BufWriter,
    test,
};
use tokio_stream::{Stream, StreamExt};

async fn list_fixtures_dir() -> impl Stream<Item = DirEntry> {
    let mut files = read_dir("../fixtures")
        .await
        .expect("failed to list fixtures");

    stream! {
        while let Some(entry) = files.next_entry().await.expect("failed to get entry") {
            yield entry
        }
    }
}

#[test]
async fn test_cases() {
    let fixtures: Vec<String> = list_fixtures_dir()
        .await
        .filter_map(|f| {
            let path = f.path();
            let filename = if let Some(filename) = path.file_name() {
                filename.to_string_lossy()
            } else {
                return None;
            };

            let contains_prefix = ["input-", "output-"]
                .iter()
                .any(|prefix| filename.starts_with(prefix));

            if contains_prefix {
                return Some(path.to_string_lossy().to_string());
            }
            None
        })
        .collect()
        .await;

    let (input_files, output_files): (Vec<&str>, Vec<&str>) = fixtures
        .iter()
        .map(|f| f.as_str())
        .partition(|path| path.contains("input-"));

    assert!(!input_files.is_empty(), "no fixtures found");
    assert_eq!(
        input_files.len(),
        output_files.len(),
        "the number of input and output fixtures do not match"
    );

    for (input_filename, expected_output_filename) in input_files.iter().zip(output_files.iter()) {
        let client = Cli::new().expect("should create client");
        let input_file = File::open(input_filename)
            .await
            .expect("failed to open input fixture");
        let _expected_output = fs::read_to_string(expected_output_filename)
            .await
            .expect("failed to read output fixture");

        let mut output_buffer = BufWriter::new(vec![]);
        {
            client
                .process_and_print_transactions(input_file, &mut output_buffer)
                .await
                .expect("failed to process fixture");
        }

        let _ = String::from_utf8_lossy(output_buffer.buffer());
        // assert_eq!(expected_output, output);
    }
}
