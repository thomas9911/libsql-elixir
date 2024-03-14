defmodule Libsql.Connection do
  defstruct [:connection]

  def query(%__MODULE__{} = conn, statement, params) do
    Libsql.Native.query_on_conn(conn, statement, params)
  end
end
