# @TODO make it a small CLI
import pandas as pd
df = pd.read_csv('hospital_new_cases.csv', ';')
df.columns = [c.lower() for c in df.columns] 

from sqlalchemy import create_engine
engine = create_engine('postgresql://[username]:[password]@[host]:[port]/[dbname]')

df.to_sql(
  "cases",
  engine,
  if_exists="replace"
)
