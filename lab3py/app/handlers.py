import app.keyboard as keyboards

from aiogram import F, Router
from aiogram.filters import CommandStart, Command
from aiogram.types import Message, CallbackQuery, FSInputFile

router = Router()


@router.message(CommandStart())
async def command_start(message: Message):
    await message.answer(text='Какое место хотите найти?')


@router.message()
async def process_location(msg: Message):
    await msg.answer("You want me to find " + msg.text + " for you. I can do that!")
