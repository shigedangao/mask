import pandas as pd
import wget
import toml
import os
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

def download_csv(url: str, filename: str):
  wget.download(url, './' + filename)

def import_csv_to_sql(filename: str, table_name: str):
  df = pd.read_csv(filename, ';')
  df.columns = [c.lower() for c in df.columns] 
  df.to_sql(
    table_name,
    engine,
    if_exists="replace"
  )

def import_hospital_cases():
  download_csv(
    'https://www.data.gouv.fr/fr/datasets/r/08c18e08-6780-452d-9b8c-ae244ad529b3',
    'hospitalization_by_region.csv'
  )
  import_csv_to_sql('hospitalization_by_region.csv', 'hospitalization')

def import_hospital_new_cases():
  download_csv(
    'https://www.data.gouv.fr/fr/datasets/r/6fadff46-9efd-4c53-942a-54aca783c30c',
    'hospitalization_new_case.csv'
  )
  import_csv_to_sql('hospitalization_new_case.csv', 'cases')

def main():
  import_hospital_cases()
  import_hospital_new_cases()

if __name__ == "__main__":
  main()
