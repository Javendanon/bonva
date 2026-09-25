# Parse source only: do not expand macros or evaluate the target.
defmodule QualityAST do
  def walk({:quote, _, _}, acc, _fun), do: acc
  def walk(node, acc, fun) do
    acc = fun.(node, acc)
    children = cond do
      is_tuple(node) -> Tuple.to_list(node)
      is_list(node) -> node
      true -> []
    end
    Enum.reduce(children, acc, &walk(&1, &2, fun))
  end

  def head({:when, _, [head | _]}), do: head(head)
  def head({name, _, args}), do: {name, if(is_list(args), do: length(args), else: 0)}

  def decisions(node) do
    walk(node, 1, fn
      {op, _, args}, count when op in [:if, :unless, :and, :or, :&&, :||] and is_list(args) -> count + 1
      {op, _, args}, count when op in [:case, :cond, :receive] and is_list(args) ->
        blocks = List.last(args)
        clauses = if is_list(blocks), do: Keyword.get(blocks, :do, []), else: []
        count + if(is_list(clauses), do: max(length(clauses) - 1, 0), else: 0)
      _, count -> count
    end)
  end

  def function(node, kind, meta, signature, path) do
    {name, arity} = head(signature)
    first = Keyword.fetch!(meta, :line)
    last = walk(node, first, fn
      {_, metadata, _}, n when is_list(metadata) ->
        Enum.reduce([:line, :end, :end_of_expression, :closing], n, fn key, current ->
          case Keyword.get(metadata, key) do
            line when is_integer(line) -> max(current, line)
            details when is_list(details) -> max(current, Keyword.get(details, :line, current))
            _ -> current
          end
        end)
      _, n -> n
    end)
    %{file: path, line: first, name: Atom.to_string(name), arity: arity,
      kind: Atom.to_string(kind), lines: last - first + 1, decision_indicator: decisions(node)}
  end

  def analyze(path) do
    text = File.read!(path)
    result = case Code.string_to_quoted(text, columns: true, token_metadata: true) do
      {:ok, ast} ->
        functions = walk(ast, [], fn
          {kind, meta, [signature, body]} = node, acc when kind in [:def, :defp, :defmacro, :defmacrop] and is_list(body) ->
            [function(node, kind, meta, signature, path) | acc]
          _, acc -> acc
        end)
        # This detects syntax, not a proven cost or even resolved Kernel.++.
        appends = walk(ast, [], fn
          {:++, meta, [_, _]}, acc -> [%{file: path, line: meta[:line], rule: "LIST_APPEND_REVIEW"} | acc]
          _, acc -> acc
        end)
        %{file: path, success: true, functions: Enum.reverse(functions), advisory: Enum.reverse(appends)}
      {:error, error} -> %{file: path, success: false, error: inspect(error), functions: [], advisory: []}
    end
    Map.put(result, :source_sha256, Base.encode16(:crypto.hash(:sha256, text), case: :lower))
  end
end

System.argv()
|> Enum.map(&QualityAST.analyze/1)
|> JSON.encode!()
|> IO.puts()
