defmodule Classifier do
  def label(n) when n > 0, do: :positive
  def label(0), do: :zero
  def label(n) when n < 0, do: :negative

  def copy(xs), do: Enum.reverse(Enum.reduce(xs, [], fn x, acc -> [x | acc] end))
end
