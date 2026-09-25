defmodule ClassifierTest do
  use ExUnit.Case, async: false

  test "classifies a fixed integer domain using an independent sign oracle" do
    for n <- -100..100 do
      expected = cond do
        n < 0 -> :negative
        n == 0 -> :zero
        true -> :positive
      end
      assert Classifier.label(n) == expected
    end
  end

  test "copy preserves order, multiplicity, and empty input" do
    for xs <- [[], [1], [3, 1, 3], [:a, :b, :a], Enum.to_list(-20..20)] do
      assert Classifier.copy(xs) == xs
    end
  end
end
