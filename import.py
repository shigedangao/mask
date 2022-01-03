import util
import os

hospitalization_by_region_url = 'https://www.data.gouv.fr/fr/datasets/r/08c18e08-6780-452d-9b8c-ae244ad529b3'
hopsitalization_by_new_case_url = 'https://www.data.gouv.fr/fr/datasets/r/6fadff46-9efd-4c53-942a-54aca783c30c'
pcr_test_by_region_url = 'https://www.data.gouv.fr/fr/datasets/r/001aca18-df6a-45c8-89e6-f82d689e6c01'
pcr_test_by_department_url = 'https://www.data.gouv.fr/fr/datasets/r/406c6a23-e283-4300-9484-54e78c8ae675'
positivity_rate_by_department_url = 'https://www.data.gouv.fr/fr/datasets/r/4180a181-a648-402b-92e4-f7574647afa6'
data_mix_url = 'https://raw.githubusercontent.com/etalab/data-covid19-dashboard-widgets/master/files_new/vacsi_non_vacsi_nat.csv'
unvaxx_url = 'https://raw.githubusercontent.com/etalab/data-covid19-dashboard-widgets/master/dist/sc_non_vacsi.json'
vaxx_url = 'https://raw.githubusercontent.com/etalab/data-covid19-dashboard-widgets/master/dist/sc_vacsi.json'

def import_hospital_cases():
  util.download_file(hospitalization_by_region_url, 'hospitalization_by_region.csv')
  util.import_csv_to_sql(
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
  util.download_file(hopsitalization_by_new_case_url, 'hospitalization_new_case.csv')
  util.import_csv_to_sql(
    'hospitalization_new_case.csv',
    'cases',
    {"jour": "string", "incid_hosp": int, "incid_rea": int, "incid_dc": int, "incid_rad": int}
  )

def import_pcr_test_per_region():
  util.download_file(pcr_test_by_region_url, 'pcr_test_by_region.csv')
  util.import_csv_to_sql(
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
  util.download_file(pcr_test_by_department_url, 'pcr_test_by_department.csv')
  util.import_csv_to_sql(
    'pcr_test_by_department.csv',
    'pcr_test_department',
    {"dep": "string", "jour": "string", "cl_age90": int, "pop": float, "t": int, "p": int}
  )

def import_positivity_rate_per_department_by_day():
  util.download_file(positivity_rate_by_department_url, 'positivity_rate_by_department_per_day.csv')
  util.import_csv_to_sql(
    'positivity_rate_by_department_per_day.csv',
    'positivity_rate_per_dep_by_day',
    {"dep": "string", "jour": "string", "p": int, "tx_std": float}
  )

def import_data_mix():
  util.download_file(data_mix_url, 'data_mix.csv')
  util.import_csv_to_sql(
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

def import_entry_in_icu_for_non_vaxx():
  util.download_file(unvaxx_url, 'unvaxx.json')
  util.import_json_to_db('unvaxx.json', ['france', 'values'], 'unvaxx')

def import_entry_in_icu_by_region_for_non_vaxx():
  util.download_file(unvaxx_url, 'unvaxx.json')
  util.import_json_to_db('unvaxx.json', ['france', 'values'], 'unvaxx')

def import_entry_in_icu_for_vaxx():
  util.download_file(vaxx_url, 'vaxx.json')
  util.import_json_to_db('vaxx.json', ['france', 'values'], 'vaxx')

# remove csv after import
def delete_csv():
  os.remove('hospitalization_by_region.csv')
  os.remove('hospitalization_new_case.csv')
  os.remove('pcr_test_by_region.csv')
  os.remove('pcr_test_by_department.csv')
  os.remove('positivity_rate_by_department_per_day.csv')
  os.remove('data_mix.csv')
  os.remove('unvaxx.json')
  os.remove('vaxx.json')

def main():
  import_hospital_cases()
  import_hospital_new_cases()
  import_pcr_test_per_region()
  import_pcr_test_per_department()
  import_positivity_rate_per_department_by_day()
  import_data_mix()
  import_entry_in_icu_for_non_vaxx()
  import_entry_in_icu_for_vaxx()

  delete_csv()  

if __name__ == "__main__":
  main()
