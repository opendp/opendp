# Download dataset from the [Mental Health in Tech Survey](https://www.kaggle.com/osmi/mental-health-in-tech-survey)
# Mental health in tech survey data set is an open data set licensed under CC BY-SA 4.0. 
# The data consists in 27 questions, answered by 1,259 volunteers.

import os
import pandas as pd

gender_dict = {
    "cis male": "1",
    "male leaning androgynous": "0",
    "non-binary": "0",
    "Male ": "1",
    "fluid": "0",
    "Cis Man": "1",
    "Make": "1",
    "Nah": "0",
    "Cis Female": "2",
    "something kinda male?": "0",
    "maile": "1",
    "Mal": "1",
    "Enby": "0",
    "Trans-female": "0",
    "Female ": "2",
    "Androgyne": "0",
    "Genderqueer": "0",
    "Male-ish": "0",
    "cis-female/femme": "2",
    "Neuter": "0",
    "Agender": "0",
    "msle": "1",
    "Female": "2",
    "woman": "2",
    "Male": "1",
    "Malr": "1",
    "M": "1",
    "Femake": "2",
    "All": "0",
    "Woman": "2",
    "Man": "1",
    "queer": "0",
    "Mail": "1",
    "Cis Male": "1",
    "Female (cis)": "2",
    "Trans woman": "2",
    "female": "2",
    "m": "1",
    "p": "0",
    "Male (CIS)": "1",
    "f": "2",
    "ostensibly male, unsure what that really means": "0",
    "F": "2",
    "femail": "2",
    "Female (trans)": "0",
    "A little about you": "0",
    "queer/she/they": "0",
    "male": "1",
    "Guy (-ish) ^_^": "0",
}

binary_dict = {"Yes": "1", "No": "0"}

def map_country(x):
    if x == 'United States':
        return '1'
    if x == 'United Kingdom':
        return '2'
    if x == 'Canada':
        return '3'
    else:
        return '0'


# Save the downloaded csv file, and load to the notebook
# NOTE: The filepath variable must be updated to reflect
#       the actual location of your survey file
filepath = os.environ["HOME"] + "/Downloads/survey.csv"
survey = pd.read_csv(filepath)

# Exclude all participants under 21 and 100 years or older
survey = survey[(survey.Age > 21) & (survey.Age < 100)]

# create age buckets
survey.loc[(survey.Age < 30), "age"] = "0"
survey.loc[(survey.Age >= 30) & (survey.Age < 40), "age"] = "1"
survey.loc[(survey.Age >= 40) & (survey.Age < 50), "age"] = "2"
survey.loc[(survey.Age >= 50) & (survey.Age < 60), "age"] = "3"
survey.loc[(survey.Age >= 60), "age"] = "4"

# create mappings for the other variables
survey["gender"] = survey.Gender.map(gender_dict)
survey["country"] = survey.Country.apply(lambda x: map_country(x))
survey["treatment"] = survey.treatment.map(binary_dict)
survey["family_history"] = survey.family_history.map(binary_dict)
survey["remote_work"] = survey.remote_work.map(binary_dict)

# save the preprocessed data
survey.to_csv('data.csv', index=False, header=False)