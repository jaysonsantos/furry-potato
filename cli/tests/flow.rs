use async_stream::stream;
use csv_async::Trim;
use krak_it::{setup_instrumentation, Cli};
use tokio::{
    fs::{read_dir, DirEntry, File},
    io::BufWriter,
    test,
};
use tokio_stream::{Stream, StreamExt};
use tracing::{info, info_span};
use transaction::client::ClientPosition;

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
    setup_instrumentation();
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

    let (mut input_files, mut output_files): (Vec<&str>, Vec<&str>) = fixtures
        .iter()
        .map(|f| f.as_str())
        .partition(|path| path.contains("input-"));

    assert!(!input_files.is_empty(), "no fixtures found");
    assert_eq!(
        input_files.len(),
        output_files.len(),
        "the number of input and output fixtures do not match"
    );

    input_files.sort();
    output_files.sort();

    for (case_number, (input_filename, expected_output_filename)) in
        input_files.iter().zip(output_files.iter()).enumerate()
    {
        let case_number = format!("{:03}", case_number + 1);
        let _span =
            info_span!("case", number = %case_number, input_filename, expected_output_filename)
                .entered();

        info!("Processing case");
        let client = Cli::new().expect("should create client");
        let input_file = File::open(input_filename)
            .await
            .expect("failed to open input fixture");
        let expected_output = File::open(expected_output_filename)
            .await
            .expect("failed to read output fixture");
        let mut output_deserializer = csv_async::AsyncReaderBuilder::new()
            .delimiter(b',')
            .trim(Trim::All)
            .create_deserializer(expected_output);
        let expected_output = output_deserializer
            .deserialize::<ClientPosition>()
            .collect::<Result<Vec<_>, _>>()
            .await
            .expect("failed to read output into structs");

        let output = vec![];
        let mut output_writer = BufWriter::new(output);
        {
            client
                .process_and_print_transactions(input_file, &mut output_writer)
                .await
                .expect("failed to process fixture");
        }

        let output = output_writer.into_inner();
        let mut output_deserializer = csv_async::AsyncReaderBuilder::new()
            .delimiter(b',')
            .trim(Trim::All)
            .create_deserializer(&output[..]);
        let output = output_deserializer
            .deserialize::<ClientPosition>()
            .collect::<Result<Vec<_>, _>>()
            .await
            .expect("failed to read output into structs");
        assert_eq!(expected_output.len(), output.len());
        for (i, (expected, output)) in expected_output.iter().zip(output.iter()).enumerate() {
            assert_eq!(
                expected,
                output,
                "client position on line {} does not match",
                i + 2
            );
        }
    }
}
