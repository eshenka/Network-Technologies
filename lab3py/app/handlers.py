import app.keyboard as keyboards

from aiogram.types import ReplyKeyboardMarkup, KeyboardButton, InlineKeyboardMarkup, InlineKeyboardButton
from aiogram.utils.keyboard import ReplyKeyboardBuilder, InlineKeyboardBuilder

from aiogram import F, Router
from aiogram.filters import CommandStart, Command
from aiogram.types import Message, CallbackQuery, FSInputFile

import requests

from config import GRAPHHOPPER_API_KEY

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
        places.append([InlineKeyboardButton(text=name, callback_data="place-btn-chosen")])

    places_btns = InlineKeyboardMarkup(inline_keyboard=places)

    await msg.answer(text="Пожалуйста, выберите подходящее место", reply_markup=places_btns)
