from fastapi import FastAPI, Response, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import gymnasium
import json
import numpy as np
import gzip
import requests
import time
import msgpack
import io
from numpy.typing import NDArray
from PIL import Image


app = FastAPI()
rust_host = "http://127.0.0.1:5000"

env = None

# CORSの設定
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

class InitData(BaseModel):
    env_name: str
    seed: int

class StepData(BaseModel):
    action: int

class ResponseData(BaseModel):
    output: list

def rgb_to_rgba(rgb_array: NDArray):
    # Convert RGB to RGBA
    a_array = np.full((rgb_array.shape[0], rgb_array.shape[1], 1), 255, dtype=np.uint8)
    rgba_array = np.concatenate([rgb_array, a_array], axis=2)
    rgba_array = rgba_array.reshape(-1)
    return rgba_array

@app.post("/init")
async def init_env(init_data: InitData):
    global env
    try:
        env = gymnasium.make(init_data.env_name, render_mode="rgb_array")
        observation, info = env.reset(seed=init_data.seed)
        return {"message": "Environment is created."}
    except Exception as e:
        raise HTTPException(status_code=400, detail="Environment creation failed.")
    
@app.post("/step")
async def step(step_data: StepData):
    if env is None:
        raise HTTPException(status_code=400, detail="Environment is not created.")
    
    try:
        observation, reward, terminated, truncated, info = env.step(step_data.action)
    except Exception as e:
        raise HTTPException(status_code=400, detail="Step failed.")
    done = terminated or truncated
    if done:
        observation, info = env.reset()
        
    return {"done": done}

@app.get("/render")
async def render():
    if env is None:
        raise HTTPException(status_code=400, detail="Environment is not created.")
    rgb = env.render()
    rgb = np.array(rgb, dtype=np.uint8)
    width = rgb.shape[1]
    height = rgb.shape[0]
    rgba = rgb_to_rgba(rgb)
    rgba = rgba.reshape(600, 400, 4)
    return Response(content=rgba.tobytes(), media_type="application/octet-stream")

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="127.0.0.1", port=8000)