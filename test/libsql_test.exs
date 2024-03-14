defmodule LibsqlTest do
  use ExUnit.Case
  doctest Libsql

  @drop_table """
  drop table if exists movies;
  """

  @create_table """
  create table if not exists movies (
        id integer primary key,
        title varchar(255),
        year integer default 2023,
        rated varchar(20),
        run_time varchar(20) default '120 min',
        plot text,
        genres varchar(255),
        poster varchar(255),
        watched boolean default false
  )
  """

  @insert_data """
  insert into movies (title, rated, plot, watched) values (?, ?, ?, ?)
  """

  test "greets the world" do
    assert Libsql.hello() == :world
  end

  test "new local db" do
    {:ok, db} = Libsql.Database.new_local("tmp.db")
    {:ok, conn} = Libsql.Database.connection(db)

    {:ok, %Libsql.Result{last_insert_id: 0}} = Libsql.Connection.query(conn, @drop_table, [])

    {:ok, %Libsql.Result{last_insert_id: 0}} = Libsql.Connection.query(conn, @create_table, [])

    {:ok, %Libsql.Result{last_insert_id: 1}} =
      Libsql.Connection.query(conn, @insert_data, [
        "logging a rocket",
        "PG-13",
        "faced with the prospect of having to workaround a solution to data distribution, a software engineer forks his favorite embedded database",
        1
      ])

    {:ok, %Libsql.Result{last_insert_id: 2}} =
      Libsql.Connection.query(conn, @insert_data, [
        "Cheese its",
        "PG-13",
        "Cheese its everywhere!!",
        1
      ])

    {:ok,
     %Libsql.Result{
       last_insert_id: 2,
       num_rows: 2,
       columns: [
         "id",
         "title",
         "year",
         "rated",
         "run_time",
         "plot",
         "genres",
         "poster",
         "watched"
       ],
       rows: [
         [
           1,
           "logging a rocket",
           2023,
           "PG-13",
           "120 min",
           "faced with the prospect of having to workaround a solution to data distribution, a software engineer forks his favorite embedded database",
           nil,
           nil,
           1
         ],
         [2, "Cheese its", 2023, "PG-13", "120 min", "Cheese its everywhere!!", nil, nil, 1]
       ]
     }} = Libsql.Connection.query(conn, "select * from movies limit 100", [])
  end
end
