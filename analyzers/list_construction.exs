# A deliberately narrow recognizer. Never load the target module or expand its macros.
defmodule ListConstruction do
  def variable?({name, _, context}), do: is_atom(name) and name != :_ and (is_atom(context) or is_nil(context))
  def variable?(_), do: false
  def same?(a, b), do: variable?(a) and variable?(b) and elem(a, 0) == elem(b, 0)
  def block({:__block__, _, [one]}), do: one
  def block(other), do: other

  # Capture only the full identity-copy body, including its input parameter.
  def classify({{:., _, [{:__aliases__, _, [:Enum]}, :reduce]}, _, [input, [], {:fn, _, [{:->, _, [[item, acc], body]}]}]}, arg) do
    cond do
      not same?(input, arg) or not variable?(item) or not variable?(acc) or same?(item, acc) -> nil
      true ->
        case block(body) do
          {:++, _, [left, [right]]} -> if same?(left, acc) and same?(right, item), do: "append_singleton"
          _ -> nil
        end
    end
  end
  def classify({{:., _, [{:__aliases__, _, [:Enum]}, :reverse]}, _, [reduce]}, arg) do
    case reduce do
      {{:., _, [{:__aliases__, _, [:Enum]}, :reduce]}, _, [input, [], {:fn, _, [{:->, _, [[item, acc], body]}]}]} ->
        case block(body) do
          [{:|, _, [head, tail]}] ->
            if same?(input, arg) and variable?(item) and variable?(acc) and not same?(item, acc) and same?(head, item) and same?(tail, acc), do: "prepend_reverse"
          _ -> nil
        end
      _ -> nil
    end
  end
  def classify(_, _), do: nil

  def walk({:quote, _, _}, acc, _fun), do: acc
  def walk(node, acc, fun) do
    acc = fun.(node, acc)
    children = cond do is_tuple(node) -> Tuple.to_list(node); is_list(node) -> node; true -> [] end
    Enum.reduce(children, acc, &walk(&1, &2, fun))
  end

  # No inference about lexical bindings through aliases, imports, use or macros.
  # Reject the entire file if its top-level module can change those bindings.
  def module_body?({:defmodule, _, [{:__aliases__, _, names}, [do: _]]}), do: names not in [[:Enum], [:Kernel], [:Elixir, :Enum], [:Elixir, :Kernel]]
  def module_body?(_), do: false
  def lexical_reasons(ast) do
    initial = if module_body?(ast), do: [], else: ["requires_one_literal_module"]
    walk(ast, initial, fn
      {op, _, _}, acc when op in [:alias, :import, :require, :use, :defmacro, :defmacrop, :defdelegate, :@] -> ["unverified_lexical_environment" | acc]
      _, acc -> acc
    end) |> Enum.uniq() |> Enum.sort()
  end

  def definitions({:defmodule, _, [_, [do: body]]}) do
    nodes = case body do {:__block__, _, nodes} -> nodes; node -> [node] end
    nodes
  end
  def definitions(_), do: []

  def signature({:when, _, [head | _]}), do: signature(head)
  def signature({name, _, args}) when is_atom(name) and is_list(args), do: {name, length(args)}
  def signature(_), do: nil

  def detect(path) do
    text = File.read!(path)
    sha = Base.encode16(:crypto.hash(:sha256, text), case: :lower)
    case Code.string_to_quoted(text, columns: true, token_metadata: true) do
      {:error, reason} -> {%{file: path, source_sha256: sha, success: false, error: inspect(reason), sites: [], unsupported: []}, []}
      {:ok, ast} ->
        reasons = lexical_reasons(ast)
        nodes = definitions(ast)
        # Other executable module-level constructs might redefine bindings or generate clauses.
        simple_module = Enum.all?(nodes, fn
          {kind, _, [head, body]} when kind in [:def, :defp] and is_list(body) ->
            case signature(head) do
              {name, _} when name != :++ -> true
              _ -> false
            end
          _ -> false
        end)
        reasons = if simple_module, do: reasons, else: Enum.uniq(reasons ++ ["unsupported_module_construct"])
        frequencies = Enum.frequencies_by(nodes, fn
          {_, _, [head, _]} -> signature(head)
          _ -> nil
        end)
        {sites, funs} = Enum.reduce(nodes, {[], []}, fn
          {kind, meta, [{name, _, [arg]}, [do: body]]}, {sites, funs} when kind in [:def, :defp] ->
            strategy = classify(block(body), arg)
            if strategy && reasons == [] && frequencies[{name, 1}] == 1 do
              # Only the recognized AST expression is evaluated in a fresh default environment.
              # No project code, mix.exs, module body or arbitrary callback is loaded.
              {fun, []} = Code.eval_quoted({:fn, [], [{:->, [], [[arg], body]}]})
              site = %{file: path, line: meta[:line], column: meta[:column], name: Atom.to_string(name), arity: 1,
                pattern: strategy, source_sha256: sha, expression: Macro.to_string(body),
                premises: %{single_unguarded_clause: true, empty_accumulator: true, input_is_parameter: true,
                  identity_element: true, fixed_singleton_append_or_prepend: true, default_bindings: true},
                domain: "proper_lists_only"}
              {[site | sites], [fun | funs]}
            else
              {sites, funs}
            end
          _, acc -> acc
        end)
        append_sites = walk(ast, [], fn
          {:++, meta, [_, _]}, acc -> [%{file: path, line: meta[:line], column: meta[:column], reason: "append_syntax_without_verified_full_identity_copy", lexical_reasons: reasons} | acc]
          _, acc -> acc
        end) |> Enum.reverse()
        # Unsupported occurrences are retained even when another function is verified.
        unsupported = Enum.reject(append_sites, fn s ->
          Enum.any?(sites, fn site -> site.pattern == "append_singleton" and append_in_definition?(nodes, site, s) end)
        end)
        {%{file: path, source_sha256: sha, success: true, sites: Enum.reverse(sites), unsupported: unsupported}, Enum.reverse(funs)}
    end
  end
  def append_in_definition?(nodes, site, occurrence) do
    Enum.any?(nodes, fn
      {_, meta, _} = node -> meta[:line] == site.line and meta[:column] == site.column and walk(node, false, fn
        {:++, meta, _}, found -> found or (meta[:line] == occurrence.line and meta[:column] == occurrence.column)
        _, found -> found
      end)
      _ -> false
    end)
  end

  def words(_, 0), do: [[]]
  def words(alphabet, n), do: for(x <- alphabet, tail <- words(alphabet, n - 1), do: [x | tail])
  def cases(config) do
    Enum.flat_map(0..config["domain"]["max_length"], &words(config["domain"]["alphabet"], &1)) ++ config["additional_cases"]
  end
  def reference(xs), do: Enum.reverse(Enum.reduce(xs, [], fn x, acc -> [x | acc] end))

  # Executable linked-list model: count constructors while actually constructing lists.
  # This is not a measurement of BEAM allocations or bytes.
  def append_count([], right), do: {right, 0}
  def append_count([head | tail], right) do
    {rest, copies} = append_count(tail, right)
    {[head | rest], copies + 1}
  end
  def reverse_count([], acc, count), do: {acc, count}
  def reverse_count([head | tail], acc, count), do: reverse_count(tail, [head | acc], count + 1)
  def model(xs, "append_singleton") do
    {result, visits, constructors} = Enum.reduce(xs, {[], 0, 0}, fn x, {acc, visits, constructors} ->
      {next, copies} = append_count(acc, [x])
      {next, visits + copies, constructors + copies + 1}
    end)
    %{result_matches: result === xs, input_visits: length(xs), extra_spine_visits: visits, constructors: constructors}
  end
  def model(xs, "prepend_reverse") do
    {reversed, count} = Enum.reduce(xs, {[], 0}, fn x, {acc, count} -> {[x | acc], count + 1} end)
    {result, visits} = reverse_count(reversed, [], 0)
    %{result_matches: result === xs, input_visits: count, extra_spine_visits: visits, constructors: count + visits}
  end

  def sample(fun, input) do
    parent = self()
    {pid, monitor} = spawn_monitor(fn ->
      :erlang.garbage_collect()
      {:reductions, before} = Process.info(self(), :reductions)
      start = System.monotonic_time()
      result = fun.(input)
      elapsed = System.monotonic_time() - start
      {:reductions, after_count} = Process.info(self(), :reductions)
      send(parent, {self(), %{elapsed_ns: System.convert_time_unit(elapsed, :native, :nanosecond),
        reductions: after_count - before, matches_identity: result === input}})
    end)
    receive do
      {^pid, result} ->
        receive do {:DOWN, ^monitor, :process, ^pid, :normal} -> result end
      {:DOWN, ^monitor, :process, ^pid, reason} -> raise "Sample failed: #{inspect(reason)}"
    after
      10_000 -> Process.exit(pid, :kill); raise "Sample timeout"
    end
  end

  def experiment(site, fun, config) do
    # Use the same evaluator for both variants. Comparing an erl_eval closure with
    # a compiled module function would measure execution-mode overhead as well.
    {alternative_fun, []} = Code.eval_quoted(quote do
      fn xs -> Enum.reverse(Enum.reduce(xs, [], fn x, acc -> [x | acc] end)) end
    end)
    checks = Enum.map(cases(config), fn input ->
      actual = fun.(input)
      alternative = alternative_fun.(input)
      %{input: input, actual: actual, alternative: alternative, passed: actual === input and alternative === input}
    end)
    series = Enum.map(config["benchmark"]["sizes"], fn n ->
      input = Enum.to_list(1..n)
      for _ <- 1..config["benchmark"]["warmup"], do: (sample(fun, input); sample(alternative_fun, input))
      samples = Enum.map(1..config["benchmark"]["samples"], fn i ->
        if rem(i, 2) == 1 do
          a = sample(fun, input); b = sample(alternative_fun, input)
          %{order: ["actual", "alternative"], actual: a, alternative: b}
        else
          b = sample(alternative_fun, input); a = sample(fun, input)
          %{order: ["alternative", "actual"], actual: a, alternative: b}
        end
      end)
      %{n: n, input: input, samples: samples, model: %{actual: model(input, site.pattern), alternative: model(input, "prepend_reverse")}}
    end)
    Map.merge(site, %{checks: checks, series: series})
  end

  def run(config, paths) do
    detected = Enum.map(paths, &detect/1)
    sites = Enum.flat_map(detected, fn {r, funs} -> Enum.zip(r.sites, funs) end)
    measured = sites |> Enum.take(config["limits"]["max_sites"]) |> Enum.map(fn {site, fun} -> experiment(site, fun, config) end)
    %{schema_version: "1.0", detector: "list_construction_v1", files: Enum.map(detected, &elem(&1, 0)),
      experiments: measured, omitted_sites: max(length(sites) - length(measured), 0),
      runtime: %{elixir: System.version(), otp: System.otp_release()}}
  end
end

if System.get_env("QUALITY_LIST_LIBRARY") != "1" do
  [config_path | paths] = System.argv()
  config = config_path |> File.read!() |> JSON.decode!()
  ListConstruction.run(config, paths) |> JSON.encode!() |> IO.puts()
end
