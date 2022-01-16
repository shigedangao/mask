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
hospital_data_per_department_url = 'https://www.data.gouv.fr/fr/datasets/r/63352e38-d353-4b54-bfd1-f1b3ee1cabd7'
pcr_test_country_url = 'https://www.data.gouv.fr/fr/datasets/r/dd0de5d9-b5a5-4503-930a-7b08dc0adc7c'

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
  os.remove('hospitalization_by_region.csv')

def import_hospital_new_cases():
  util.download_file(hopsitalization_by_new_case_url, 'hospitalization_new_case.csv')
  util.import_csv_to_sql(
    'hospitalization_new_case.csv',
    'cases',
    {"jour": "string", "incid_hosp": int, "incid_rea": int, "incid_dc": int, "incid_rad": int}
  )
  os.remove('hospitalization_new_case.csv')

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
  os.remove('pcr_test_by_region.csv')

def import_pcr_test_per_department():
  util.download_file(pcr_test_by_department_url, 'pcr_test_by_department.csv')
  util.import_csv_to_sql(
    'pcr_test_by_department.csv',
    'pcr_test_department',
    {"dep": "string", "jour": "string", "cl_age90": int, "pop": float, "t": int, "p": int}
  )
  os.remove('pcr_test_by_department.csv')

def import_positivity_rate_per_department_by_day():
  util.download_file(positivity_rate_by_department_url, 'positivity_rate_by_department_per_day.csv')
  util.import_csv_to_sql(
    'positivity_rate_by_department_per_day.csv',
    'positivity_rate_per_dep_by_day',
    {"dep": "string", "jour": "string", "p": int, "tx_std": float}
  )
  os.remove('positivity_rate_by_department_per_day.csv')

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
      "effectif": float
    }
  )
  os.remove('data_mix.csv')

def import_entry_in_icu_for_non_vaxx():
  util.download_file(unvaxx_url, 'unvaxx.json')
  util.import_json_to_db('unvaxx.json', ['france', 'values'], 'unvaxx')
  os.remove('unvaxx.json')

def import_entry_in_icu_for_vaxx():
  util.download_file(vaxx_url, 'vaxx.json')
  util.import_json_to_db('vaxx.json', ['france', 'values'], 'vaxx')
  os.remove('vaxx.json')

def import_hospital_data_per_department():
  util.download_file(hospital_data_per_department_url, 'hospital_dep.csv')
  util.import_csv_to_sql(
    'hospital_dep.csv',
    'hospital_dep',
    {
      "dep": "string",
      "sexe": int,
      "jour": "string",
      "hosp": int,
      "rea": int,
      "rad": int,
      "dc": int,
      "ssr_usld": float,
      "hospconv": float,
      "autres": float
    }
  )
  os.remove('hospital_dep.csv')

def import_positivity_rate_country():
  util.download_file(pcr_test_country_url, 'pcr_country.csv')
  util.import_csv_to_sql(
    'pcr_country.csv',
    'pcr_country',
    {
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
  os.remove('pcr_country.csv')

def main():
  import_hospital_cases()
  import_hospital_new_cases()
  import_pcr_test_per_region()
  import_pcr_test_per_department()
  import_positivity_rate_per_department_by_day()
  import_data_mix()
  import_entry_in_icu_for_non_vaxx()
  import_entry_in_icu_for_vaxx()
  import_hospital_data_per_department()
  import_positivity_rate_country()

if __name__ == "__main__":
  main()
