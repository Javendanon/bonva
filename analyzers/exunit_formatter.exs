defmodule QualityExUnitFormatter do
  use GenServer

  def init(_opts), do: {:ok, %{total: 0, passed: 0, failed: 0, skipped: 0, excluded: 0}}

  def handle_cast({:test_finished, test}, counts) do
    key = case test.state do
      nil -> :passed
      {:excluded, _} -> :excluded
      {:skipped, _} -> :skipped
      _ -> :failed
    end
    {:noreply, counts |> Map.update!(:total, &(&1 + 1)) |> Map.update!(key, &(&1 + 1))}
  end

  def handle_cast({:suite_finished, _times}, counts) do
    File.write!(System.fetch_env!("QUALITY_TEST_RESULT"), JSON.encode!(counts))
    {:noreply, counts}
  end

  def handle_cast(_, counts), do: {:noreply, counts}
end
