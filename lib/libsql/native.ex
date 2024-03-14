defmodule Libsql.Native do
  use Rustler, otp_app: :libsql, crate: "libsql_native"

  # When your NIF is loaded, it will override this function.
  def add(_a, _b), do: nif_error()

  def new_local(_path), do: nif_error()

  def open_db(_db), do: nif_error()

  def query_on_conn(_conn, _statement, _params), do: nif_error()

  defp nif_error do
    :erlang.nif_error(:nif_not_loaded)
  end
end
