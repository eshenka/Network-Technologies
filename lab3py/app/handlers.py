import app.keyboard as keyboards

from aiogram.types import ReplyKeyboardMarkup, KeyboardButton, InlineKeyboardMarkup, InlineKeyboardButton
from aiogram.utils.keyboard import ReplyKeyboardBuilder, InlineKeyboardBuilder

from aiogram import F, Router
from aiogram.filters import CommandStart, Command
from aiogram.types import Message, CallbackQuery, FSInputFile

import requests

from config import GRAPHHOPPER_API_KEY
from config import WEATHER_API_KEY
from config import OPENTRIPMAP_API_KEY

router = Router()

@router.message(CommandStart())
async def command_start(message: Message):
    await message.answer(text='Какое место хотите найти?')


@router.message()
async def process_location(msg: Message):
    # await msg.answer("You want me to find " + msg.text + " for you. I can do that!")
    url = "https://graphhopper.com/api/1/geocode"

    query = {
        "q": msg.text,
        "limit": "10",
        "provider": "default",
        "key": GRAPHHOPPER_API_KEY
    }

    response = requests.get(url, params=query)
    answer = response.json()['hits']

    places = []
    for place in answer:
        name = "{}, страна: {}".format(place['name'], place['country'])
        places.append([InlineKeyboardButton(text=name, callback_data="{} {}".format(place['point']['lat'], place['point']['lng']))])

    places_btns = InlineKeyboardMarkup(inline_keyboard=places)

    await msg.answer(text="Пожалуйста, выберите подходящее место", reply_markup=places_btns)


@router.callback_query()
async def location_description(callback: CallbackQuery):
    (lat, lon) = callback.data.split(" ")

    # print(lat, lon)
    #
    # weather_url = "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}".format(lat, lon, WEATHER_API_KEY)
    #
    # weather_resp = requests.get(weather_url).json()
    #
    # print(weather_resp)

    # await callback.message.answer(text="Temperature in this place is {}".format(weather_resp['temp']))

    sights = []
    default_description = "Очень хорошее место!"

    opentripmap_url = "https://api.opentripmap.com/0.1/en/places/radius?apikey={}".format(OPENTRIPMAP_API_KEY)
    opentripmap_query = {
        "lang": "ru",
        "radius": "500",
        "lat": lat,
        "lon": lon,
        "limit": "10"
    }

    opentripmap_response = requests.get(opentripmap_url, params=opentripmap_query).json()['features']

    for sight in opentripmap_response:
        if sight['properties']['name'] == '':
            continue

        xid = sight['properties']['xid']

        url = "https://api.opentripmap.com/0.1/en/places/xid/{}?apikey={}".format(xid, OPENTRIPMAP_API_KEY)
        query = {
            "lang": "ru",
            "xid": xid
        }

        sight_description = requests.get(url, params=query).json()

        if 'wikipedia_extracts' in sight_description:
            sights.append(
                {'name': sight['properties']['name'], 'descr': sight_description['wikipedia_extracts']['text']})
        else:
            sights.append(
                {'name': sight['properties']['name'], 'descr': default_description})

    reply_message_text = ""
    for sight in sights:
        reply_message_text += "Название: {}\nОписание: {}\n\n".format(sight['name'], sight['descr'])

    if reply_message_text == "":
        reply_message_text = "Не найдено достопримечательностей поблизости"

    await callback.message.answer(text=reply_message_text)
