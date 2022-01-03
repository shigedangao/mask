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

def import_csv_to_sql(filename: str, table_name: str, dic):
  print("\nprocess "+filename)
  df = pd.read_csv(filename, ';', dtype=dic)
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
  import_csv_to_sql(
    'hospitalization_by_region.csv',
    'hospitalization',
    {
      "reg": int,
      "cl_age90": int,
      "hosp": int,
      "rea": int,
      "hospconv": float,
      "ssr_usld": float,
      "autres": float,
      "rad": int,
      "dc": int,
      "jour": "string"
    }
  )

def import_hospital_new_cases():
  download_csv(
    'https://www.data.gouv.fr/fr/datasets/r/6fadff46-9efd-4c53-942a-54aca783c30c',
    'hospitalization_new_case.csv'
  )
  import_csv_to_sql(
    'hospitalization_new_case.csv',
    'cases',
    {"jour": "string", "incid_hosp": int, "incid_rea": int, "incid_dc": int, "incid_rad": int}
  )

def import_pcr_test_per_region():
  download_csv(
    'https://www.data.gouv.fr/fr/datasets/r/001aca18-df6a-45c8-89e6-f82d689e6c01',
    'pcr_test_by_region.csv'
  )
  import_csv_to_sql(
    'pcr_test_by_region.csv',
    'pcr_test_region',
    {
      "reg": int,
      "jour": "string",
      "P_f": int,
      "P_h": int,
      "P": int,
      "T": int,
      "T_f": int,
      "T_h": int,
      "cl_age90": int,
      "pop": float
    }
  )

def import_pcr_test_per_department():
  download_csv(
    'https://www.data.gouv.fr/fr/datasets/r/406c6a23-e283-4300-9484-54e78c8ae675',
    'pcr_test_by_department.csv'
  )
  import_csv_to_sql(
    'pcr_test_by_department.csv',
    'pcr_test_department',
    {"dep": "string", "jour": "string", "cl_age90": int, "pop": float, "t": int, "p": int}
  )

def import_positivity_rate_per_department_by_day():
  download_csv(
    'https://www.data.gouv.fr/fr/datasets/r/4180a181-a648-402b-92e4-f7574647afa6',
    'positivity_rate_by_department_per_day.csv'
  )
  import_csv_to_sql(
    'positivity_rate_by_department_per_day.csv',
    'positivity_rate_per_dep_by_day',
    {"dep": "string", "jour": "string", "p": int, "tx_std": float}
  )

def import_data_mix():
  download_csv(
    'https://raw.githubusercontent.com/etalab/data-covid19-dashboard-widgets/master/files_new/vacsi_non_vacsi_nat.csv',
    'data_mix.csv'
  )
  import_csv_to_sql(
    'data_mix.csv',
    'data_mix',
    {
      "date": "string",
      "vac_statut": "string",
      "nb_PCR": float,
      "nb_PCR_sympt": float,
      "nb_PCR+": float,
      "nb_PCR+_sympt": float,
      "HC": float,
      "HC_PCR+": float,
      "SC": float,
      "SC_PCR+": float,
      "DC": float,
      "DC_PCR+": float,
      "effectif": int
    }
  )

# remove csv after import
def delete_csv():
  os.remove('hospitalization_by_region.csv')
  os.remove('hospitalization_new_case.csv')
  os.remove('pcr_test_by_region.csv')
  os.remove('pcr_test_by_department.csv')
  os.remove('positivity_rate_by_department_per_day.csv')
  os.remove('data_mix.csv')

def main():
  import_hospital_cases()
  import_hospital_new_cases()
  import_pcr_test_per_region()
  import_pcr_test_per_department()
  import_positivity_rate_per_department_by_day()
  import_data_mix()
  delete_csv()  

if __name__ == "__main__":
  main()
