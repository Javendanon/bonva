//! AST contracts formerly exercised by the duplicate Python evaluator's tests.
use agent_quality::data;
use serde_json::{Value, json};
use std::{fs, process::Command};

fn parse(source: &str) -> Value {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("sample.ex");
    fs::write(&path, source).unwrap();
    let output = Command::new("elixir")
        .arg(data::root().join("analyzers/elixir_ast.exs"))
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    data::decode(&output.stdout).unwrap()[0].clone()
}

#[test]
#[ignore = "requires Elixir >= 1.18 on PATH"]
fn ast_distinguishes_code_from_comments_strings_and_quotes() {
    let row = parse(
        r#"defmodule Sample do
      # if x, do: a ++ b
      def text, do: "if x, do: a ++ b"
      def quoted, do: quote(do: a ++ b)
    end"#,
    );
    assert_eq!(row["advisory"], json!([]));
    let decisions: Vec<_> = row["functions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|function| function["decision_indicator"].clone())
        .collect();
    assert_eq!(decisions, vec![json!(1), json!(1)]);

    let row = parse("defmodule Sample do\n def join(a, b), do: a ++ b\nend");
    assert_eq!(row["advisory"].as_array().unwrap().len(), 1);
    assert_eq!(row["advisory"][0]["line"], 2);
    assert_eq!(row["functions"][0]["decision_indicator"], 1);
}

#[test]
#[ignore = "requires Elixir >= 1.18 on PATH"]
fn ast_preserves_clauses_guards_and_case_decisions() {
    let row = parse(
        r"defmodule Sample do
      def x(a \\ 1)
      def x(a) when is_integer(a), do: a
      def x(_), do: 0
    end",
    );
    assert_eq!(row["success"], true);
    let functions = row["functions"].as_array().unwrap();
    assert_eq!(functions.len(), 2);
    assert!(functions.iter().all(|function| function["arity"] == 1));

    let row = parse(
        "defmodule X do\n def f(x) do\n case x do\n 0 -> :a\n 1 -> :b\n _ -> :c\n end\n end\nend",
    );
    assert_eq!(row["functions"][0]["decision_indicator"], 3);
    assert_eq!(parse("defmodule Broken do def")["success"], false);
}
