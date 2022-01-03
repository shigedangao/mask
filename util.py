import wget
import toml
import os
import pandas as pd
import json
from sqlalchemy import create_engine


def set_engine():
  # use os var is db_username exist
  if "db_username" in os.environ:
    username = os.environ['db_username']
    password = os.environ['db_password']
    host     = os.environ['db_host']
    port     = os.environ['db_port']
    dbname   = os.environ['db_name']
  else:
    config = toml.load("config.toml")
    username = config.get("db_username")
    password = config.get("db_password")
    host     = config.get('db_host')
    port     = config.get('db_port')
    dbname   = config.get('db_name')

  return create_engine('postgresql://'+username+':'+password+'@'+host+':'+port+'/'+dbname)   

# variable engine
engine = set_engine()

def download_file(url: str, filename: str):
  wget.download(url, './' + filename)

def import_csv_to_sql(filename: str, table_name: str, dic):
  print("\nprocess "+filename)
  df = pd.read_csv(filename, ';', dtype=dic)
  df.columns = [c.lower() for c in df.columns] 
  df.to_sql(
    table_name,
    engine,
    if_exists="replace"
  )

def import_json_to_db(filename: str, path, table_name: str):
  print("\nprocessing "+filename)
  with open(filename) as f:
    data = json.loads(f.read())

  df = pd.json_normalize(data, path)
  df.to_sql(
    table_name,
    engine,
    if_exists="replace"
  )
