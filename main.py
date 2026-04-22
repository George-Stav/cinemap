# What information do I need?
# - all movies played in each cinema location listed
# - many-to-many relationship

# How do I get that info?
# - listed by cinema             -> <div class="item card-item">
# - one or more rooms per cinema -> <div class="grid schedule-grid">
# - one or more movies per room  -> <div class="item schedule-item">
# - every movie has a schedule   -> <span class="time">

from bs4 import BeautifulSoup
from pprint import pprint
import pandas as pd

html_doc = open("data/all-cinemas-gr.html", "r")
soup = BeautifulSoup(html_doc, "html.parser")

columns=["cinema", "room", "movie", "schedule"]
data = {key: [] for key in columns}

for cinema_element in soup.find_all("div", class_="item card-item"):
    # Organise in a csv
    # cinema | room | movie | schedule
    cinema = cinema_element.h2.text.strip()
    # if <div class="schedule-grid-title"> exists then use that for the room name, else "default"
    for room_element in cinema_element.find_all("div", class_="grid schedule-grid"):
        room_title_element = room_element.find("div", class_="schedule-grid-title")
        room = "default" if not room_title_element else room_title_element.span.text.strip()
        for movie_element in room_element.find_all("div", class_="item schedule-item"):
            movie = movie_element.h3.text.strip()
            schedule = "".join([span.text for span in movie_element.find_all("span", class_="time")])
            data["cinema"] = cinema
            data["room"] = room
            data["movie"] = movie
            data["schedule"] = schedule

df = pd.DataFrame(data=data)
df.to_csv("data/athinorama.csv", index=False, encoding="utf-8")
