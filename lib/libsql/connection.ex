defmodule Libsql.Connection do
  defstruct [:connection]

  @default_timeout 5_000

  def query(%__MODULE__{} = conn, statement, params, timeout \\ @default_timeout) do
    case Libsql.Native.query_on_conn_callback(conn, statement, params, self()) do
      {:ok, _} ->
        receive do
          data -> data
        after
          timeout -> {:error, :timeout}
        end

      err ->
        err
    end
  end
end
