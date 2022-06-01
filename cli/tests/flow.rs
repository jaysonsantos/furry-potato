use color_eyre::{eyre::WrapErr, Result};
use csv_async::Trim;
use krak_it::{setup_instrumentation, Cli};
use tokio::{fs::File, io::BufWriter, test};
use tokio_stream::StreamExt;
use transaction::client::ClientPosition;

macro_rules! positive_test_cases {
    ($input_file:ident) => {
        #[test]
        async fn $input_file() {
            let input_filename = format!("../fixtures/{}.csv", stringify!($input_file));
            let output_filename = format!("../fixtures/{}-output.csv", stringify!($input_file));
            setup_instrumentation();
            test_case_impl(&input_filename, &output_filename).await.unwrap();
        }
    };
    ($input_file:ident, $($input_files:ident),+) => {
        positive_test_cases!($input_file);
        positive_test_cases!($($input_files),+);
    }
}

positive_test_cases!(
    big_decimals,
    chargeback,
    open_dispute,
    resolve_dispute,
    shifted_columns
);

macro_rules! negative_test_cases {
    ($input_file:ident) => {
        #[test]
        async fn $input_file() {
            let input_filename = format!("../fixtures/{}.csv", stringify!($input_file));
            let output_filename = format!("../fixtures/{}-output.csv", stringify!($input_file));
            setup_instrumentation();
            let err = test_case_impl(&input_filename, &output_filename).await.unwrap_err();
            let expected_error = tokio::fs::read_to_string(output_filename).await.unwrap();
            assert_eq!(format!("{}", err.root_cause()), expected_error.trim());
        }
    };
    ($input_file:ident, $($input_files:ident),+) => {
        negative_test_cases!($input_file);
        negative_test_cases!($($input_files),+);
    }
}

negative_test_cases!(multiple_resolve_dispute);

async fn test_case_impl(input_filename: &str, output_filename: &str) -> Result<()> {
    let client = Cli::new().expect("should create client");
    let input_file = File::open(input_filename)
        .await
        .wrap_err_with(|| format!("failed to open input fixture {}", input_filename))?;
    let expected_output = File::open(output_filename)
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
            .wrap_err("failed to process fixture")?;
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
        .wrap_err("failed to read output into structs")?;
    for (i, (expected, output)) in expected_output.iter().zip(output.iter()).enumerate() {
        assert_eq!(
            expected,
            output,
            "client position on line {} does not match",
            i + 2
        );
    }
    Ok(())
}
