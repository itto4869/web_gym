from fastapi import FastAPI, Response, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import gymnasium
import json
import numpy as np
import gzip
import requests
import time

app = FastAPI()
rust_host = "http://127.0.0.1:5000"

env = gymnasium.make('CartPole-v0', render_mode="rgb_array")
observation, info = env.reset(seed=42)

# CORSの設定
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

class InputData(BaseModel):
    input: int

class ResponseData(BaseModel):
    output: list

class ConvertData(BaseModel):
    data: list
    width: int
    height: int

def square(input: int) -> int:
    return input * input

@app.get("/")
def read_root():
    return {"Hello": "World"}

@app.get("/items/{item_id}")
def read_item(item_id: int, q: str | None = None):
    return {"item_id": item_id, "q": q}

@app.post("/api/")
def read_rust_send(data: InputData): # TODO: fastapiの非同期通信とrustの並列化，websocketを使って高速化
    output = env.render()
    output = np.array(output, dtype=np.uint8)
    output = output.tolist() # type: ignore
    start_time = time.time()
    response = requests.post(f"{rust_host}/convert", json={"data": output})
    end_time = time.time()
    elapsed_time = end_time - start_time
    print(f"Time taken for Rust request: {elapsed_time:.2f} seconds")
    if response.status_code != 200:
        raise HTTPException(status_code=response.status_code, detail="Failed to forward request to actix-web server")
    
    convert_data = response.json()
    return ConvertData(**convert_data)
    

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="127.0.0.1", port=8000)