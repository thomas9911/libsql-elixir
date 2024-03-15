defmodule Libsql.Database do
  defstruct [:database]

  defdelegate new_local(path), to: Libsql.Native
  defdelegate new_remote(url, auth_token), to: Libsql.Native

  def connection(%__MODULE__{} = db) do
    Libsql.Native.open_db(db)
  end
end
