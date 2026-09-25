System.put_env("QUALITY_LIST_LIBRARY", "1")
Code.require_file("../analyzers/list_construction.exs", __DIR__)
ExUnit.start()

defmodule ListConstructionTest do
  use ExUnit.Case
  setup do
    dir = Path.join(System.tmp_dir!(), "quality-list-test-#{System.unique_integer([:positive])}")
    File.mkdir!(dir)
    on_exit(fn -> File.rm_rf!(dir) end)
    %{dir: dir}
  end

  def detect(dir, body) do
    path = Path.join(dir, "sample.ex")
    File.write!(path, "defmodule Sample do\n#{body}\nend")
    ListConstruction.detect(path)
  end

  test "recognizes only full identity-copy expressions", %{dir: dir} do
    {row, [fun]} = detect(dir, "def copy(xs), do: Enum.reduce(xs, [], fn x, acc -> acc ++ [x] end)")
    assert [%{pattern: "append_singleton"}] = row.sites
    assert fun.([1, 0, 1]) == [1, 0, 1]
    assert row.unsupported == []
    {row, [fun]} = detect(dir, "def copy(xs), do: Enum.reverse(Enum.reduce(xs, [], fn x, acc -> [x | acc] end))")
    assert [%{pattern: "prepend_reverse"}] = row.sites
    assert fun.([1, 2, 3]) == [1, 2, 3]
    {row, [_]} = detect(dir, "def copy(xs), do: Enum.reduce(xs, [], fn x, acc -> acc ++ [x] end); def other(xs), do: xs ++ [1]")
    assert length(row.sites) == 1
    assert length(row.unsupported) == 1
  end

  test "does not execute arbitrary callback or module code", %{dir: dir} do
    marker = Path.join(dir, "must-not-exist")
    for body <- [
      "File.write!(#{inspect(marker)}, \"unsafe\")\ndef copy(xs), do: Enum.reduce(xs, [], fn x, acc -> acc ++ [x] end)",
      "def copy(xs), do: Enum.reduce(xs, [], fn x, acc -> File.write!(#{inspect(marker)}, \"unsafe\"); acc ++ [x] end)"
    ] do
      {row, []} = detect(dir, body)
      assert row.sites == []
      refute File.exists?(marker)
    end
  end

  test "rejects lexical ambiguity and multiple clauses", %{dir: dir} do
    copy = "def copy(xs), do: Enum.reduce(xs, [], fn x, acc -> acc ++ [x] end)"
    for prefix <- ["alias Other, as: Enum", "import Other", "use Other", "@x 1",
                    "def copy(xs) when is_atom(xs), do: []", "def copy([]), do: []",
                    "def a ++ b, do: a"] do
      {row, []} = detect(dir, prefix <> "\n" <> copy)
      assert row.sites == []
      assert row.unsupported != []
    end
  end

  test "does not generalize append cost beyond its premises", %{dir: dir} do
    for body <- [
      "def copy(xs), do: Enum.reduce(xs, [], fn x, acc -> acc ++ x end)",
      "def copy(xs), do: Enum.reduce(xs, [], fn x, acc -> [x | acc] end)",
      "def copy(xs), do: Enum.reduce(xs, [], fn x, acc -> acc ++ [x + 1] end)",
      "def copy(xs), do: Enum.reduce(xs, [0], fn x, acc -> acc ++ [x] end)",
      "def copy(xs), do: xs ++ [1]",
      "def copy(xs), do: quote(do: Enum.reduce(xs, [], fn x, acc -> acc ++ [x] end))",
      "def copy(xs), do: \"acc ++ [x]\" # acc ++ [x]"
    ] do
      {row, []} = detect(dir, body)
      assert row.sites == []
    end
  end

  test "model executes list operations and finite domain has independent oracle" do
    config = File.read!(Path.expand("../rules/list-construction.json", __DIR__)) |> JSON.decode!()
    cases = ListConstruction.cases(config)
    assert length(cases) == 366
    for input <- cases do
      n = length(input)
      assert ListConstruction.reference(input) === input
      assert ListConstruction.model(input, "append_singleton") == %{result_matches: true,
        input_visits: n, extra_spine_visits: div(n * (n - 1), 2), constructors: div(n * (n + 1), 2)}
      assert ListConstruction.model(input, "prepend_reverse").constructors == 2 * n
    end
  end
end
