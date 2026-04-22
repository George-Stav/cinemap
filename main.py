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
import requests

OUTPUT_FILE = "data/all-cinemas-gr.html"
response = requests.get("https://www.athinorama.gr/cinema/guide/all/cinemas")
html = response.text

with open(OUTPUT_FILE, "w") as f:
    f.write(html)

soup = BeautifulSoup(html, "html.parser")
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
            data["cinema"].append(cinema)
            data["room"].append(room)
            data["movie"].append(movie)
            data["schedule"].append(schedule)

df = pd.DataFrame(data=data)
df.to_csv("data/athinorama.csv", index=False, encoding="utf-8")
