use clap::{Command, Arg};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufRead, Write};

fn main() {
    let matches = Command::new("JSON Lines to PostgreSQL Converter")
        .version("1.0")
        .author("Bruno Wego <brunowego@gmail.com")
        .about("Converts JSON Lines files to PostgreSQL")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Sets the input JSON Lines file")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Sets the output SQL file (default: schema.sql)")
                .default_value("schema.sql"),
        )
        .get_matches();

    let input_file = matches.get_one::<String>("input").expect("Input file not provided");
    let output_file = matches.get_one::<String>("output").unwrap();

    let file = File::open(input_file).expect("Unable to open input file");
    let reader = BufReader::new(file);

    let mut table_columns = HashMap::new();
    let mut insert_values = vec![];

    for line in reader.lines() {
        let line = line.expect("Unable to read line");

        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<Value>(&line) {
            Ok(json) => {
                if let Some(object) = json.as_object() {
                    let mut row_values = vec![];

                    for (key, value) in object {
                        let column_type = match value {
                            Value::Null => "NULL",
                            Value::Bool(_) => "BOOLEAN",
                            Value::Number(_) => "FLOAT",
                            Value::String(_) => "TEXT",
                            Value::Array(_) => "JSONB",
                            Value::Object(_) => "JSONB",
                        };

                        table_columns.insert(key.clone(), column_type);

                        let value_str = match value {
                            Value::Null => "NULL".to_string(),
                            Value::Bool(b) => b.to_string(),
                            Value::Number(n) => n.to_string(),
                            Value::String(s) => {
                                let escaped = s.replace("'", "''");
                                format!("'{}'", escaped)
                            },
                            Value::Array(arr) => format!("'{}'", serde_json::to_string(arr).unwrap()),
                            Value::Object(_) => format!("'{}'", serde_json::to_string(value).unwrap()),
                        };
                        row_values.push(value_str);
                    }

                    insert_values.push(row_values);
                }
            }
            Err(e) => {
                eprintln!("Invalid JSON format: {} in line: {}", e, line);
            }
        }
    }

    let create_table_sql = format!(
        "CREATE TABLE my_table (\n  {}\n);\n",
        table_columns.iter()
            .map(|(key, ty)| format!("{} {}", key, ty))
            .collect::<Vec<_>>()
            .join(",\n  ")
    );

    let insert_sqls: Vec<String> = insert_values.iter()
        .map(|values| {
            let sql = format!(
                "INSERT INTO my_table VALUES ({});",
                values.join(", ")
            );
            sql
        })
        .collect();

    let mut output = File::create(output_file).expect("Unable to create output file");
    writeln!(output, "{}", create_table_sql).expect("Unable to write to output file");

    for insert_sql in insert_sqls {
        writeln!(output, "{}", insert_sql).expect("Unable to write to output file");
    }

    println!("Schema and INSERTs written to {}", output_file);
}
