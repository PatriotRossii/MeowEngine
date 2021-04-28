import vk
from itertools import cycle
import threading
import random
import time
import codecs

def work(handler, invite_link, messages):
    user_id = handler.account.getProfileInfo()["id"]

    try:
        resp = handler.messages.joinChatByInviteLink(link=invite_link)
        chat_id = resp["chat_id"]
    except Exception:
        resp = handler.messages.getChatPreview(link=invite_link)
        chat_id = resp["preview"]["local_id"]

    counter = 0

    for message in messages:
        if counter == 3:
            break
        try:
            resp = handler.messages.send(
                chat_id=chat_id,
                message=message,
                random_id=random.randint(0, 2**32)
            )
            time.sleep(0.2)
        except Exception as e:
            print(e)
            time.sleep(0.5)
        
        counter += 1

f = open("data/tokens.txt")
tokens = [e.strip() for e in f.readlines()]
f.close()

f = open("data/invite_link.txt")
invite_link = f.read().strip()
f.close()

f = codecs.open("data/messages.txt", "r", "utf_8_sig")
messages = [e.strip() for e in f.readlines()]
for i in range(len(messages)):
    message = messages[i]
    messages[i] = message * (4096 // len(message) - 1)
messages = cycle(messages)
f.close()

handlers = []

for token in tokens:
    session = vk.Session(access_token=token)
    handlers.append(vk.API(session, v="5.130"))

threads = []
for handler in handlers:
    thread = threading.Thread(target=work, args=(handler, invite_link, messages))
    thread.start()
    threads.append(thread)

for thread in threads:
    thread.join()