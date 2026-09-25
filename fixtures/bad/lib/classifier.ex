defmodule Classifier do
  def label(n) do
    if n == 0 do
      :zero
    else
      if n == 1 do
        :positive
      else
        if n == 2 do
          :positive
        else
          if n == 3 do
            :positive
          else
            if n == 4 do
              :positive
            else
              if n == 5 do
                :positive
              else
                if n == 6 do
                  :positive
                else
                  if n == 7 do
                    :positive
                  else
                    if n == 8 do
                      :positive
                    else
                      if n == 9 do
                        :positive
                      else
                        if n > 0 do
                          :positive
                        else
                          :negative
                        end
                      end
                    end
                  end
                end
              end
            end
          end
        end
      end
    end
  end

  def copy(xs), do: Enum.reduce(xs, [], fn x, acc -> acc ++ [x] end)
end
