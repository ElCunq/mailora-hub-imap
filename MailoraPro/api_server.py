"""
Mailora AI API — MailoraPro BERT modellerini HTTP üzerinden serve eder.
Modeller: 
1. Konu Modeli v2 (12 Kategori)
2. Spam Modeli v1 (Binary)
3. Duygu Modeli Final (Pozitif/Nötr/Negatif)

Çalıştır: uvicorn api_server:app --host 0.0.0.0 --port 5000
"""

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import List
import torch
import os
import logging
from transformers import BertTokenizer, BertForSequenceClassification, pipeline
import gc

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("mailora-ai")

app = FastAPI(title="Mailora AI API", version="2.0.0")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)

device = "cuda" if torch.cuda.is_available() else "cpu"
logger.info(f"Device: {device}")

# ============ MODELLER ============
duygu_model = None
duygu_tokenizer = None
duygu_etiketler = {0: "Negatif", 1: "Nötr", 2: "Pozitif"}

konu_model = None
konu_tokenizer = None
konu_etiketler = {
    0: "is_proje", 1: "finans", 2: "alisveris", 3: "teknoloji",
    4: "pazarlama", 5: "kisisel", 6: "egitim", 7: "seyahat",
    8: "hukuk_resmi", 9: "saglik", 10: "sosyal_bildirim", 11: "spor_eglence"
}

spam_model = None
spam_tokenizer = None
spam_etiketler = {0: "Ham", 1: "Spam"}

def load_models():
    global duygu_model, duygu_tokenizer
    global konu_model, konu_tokenizer
    global spam_model, spam_tokenizer

    base_dir = os.path.dirname(os.path.abspath(__file__))

    # 1. Duygu Modeli
    yol_duygu = os.path.join(base_dir, "Modeller", "Duygu_Modeli_Final")
    if os.path.exists(yol_duygu) and os.path.exists(os.path.join(yol_duygu, "config.json")):
        logger.info(f"🎭 Duygu modeli yükleniyor: {yol_duygu}")
        duygu_tokenizer = BertTokenizer.from_pretrained(yol_duygu, local_files_only=True)
        duygu_model = BertForSequenceClassification.from_pretrained(yol_duygu, local_files_only=True).to(device)
        duygu_model.eval()
        logger.info("✅ Duygu modeli hazır!")

    # 2. Konu Modeli v3
    yol_konu = os.path.join(base_dir, "Modeller", "Konu_Modeli_v3")
    if os.path.exists(yol_konu) and os.path.exists(os.path.join(yol_konu, "config.json")):
        logger.info(f"📌 Konu modeli yükleniyor: {yol_konu}")
        konu_tokenizer = BertTokenizer.from_pretrained(yol_konu, local_files_only=True)
        konu_model = BertForSequenceClassification.from_pretrained(yol_konu, local_files_only=True).to(device)
        konu_model.eval()
        logger.info("✅ Konu modeli v2 hazır!")

    # 3. Spam Modeli
    yol_spam = os.path.join(base_dir, "Modeller", "Spam_Modeli_v1")
    if os.path.exists(yol_spam) and os.path.exists(os.path.join(yol_spam, "config.json")):
        logger.info(f"🛡️ Spam modeli yükleniyor: {yol_spam}")
        spam_tokenizer = BertTokenizer.from_pretrained(yol_spam, local_files_only=True)
        spam_model = BertForSequenceClassification.from_pretrained(yol_spam, local_files_only=True).to(device)
        spam_model.eval()
        logger.info("✅ Spam modeli hazır!")

load_models()

# ============ İSTEKLER ============
class AnalyzeRequest(BaseModel):
    id: str | None = None
    text: str

class BatchAnalyzeRequest(BaseModel):
    messages: List[AnalyzeRequest]

class TranslateRequest(BaseModel):
    text: str
    target_lang: str = "tr"  # "tr" = en-to-tr, "en" = tr-to-en

class SummarizeRequest(BaseModel):
    text: str

class NERRequest(BaseModel):
    text: str

class SmartReplyRequest(BaseModel):
    text: str

# ============ TAHMİN FONKSİYONLARI ============
def predict_single(model, tokenizer, text, etiket_map):
    inputs = tokenizer(text[:512], return_tensors="pt", truncation=True, padding=True, max_length=128).to(device)
    with torch.no_grad():
        outputs = model(**inputs)
    probs = torch.softmax(outputs.logits, dim=-1)[0].tolist()
    pred_id = torch.argmax(outputs.logits, dim=-1).item()
    scores = {etiket_map[i]: round(prob * 100, 1) for i, prob in enumerate(probs) if i in etiket_map}
    return {
        "label": etiket_map[pred_id],
        "confidence": round(max(probs) * 100, 1),
        "scores": scores
    }

def process_text(text: str):
    res = {}
    
    # Duygu
    if duygu_model and duygu_tokenizer:
        res["duygu"] = predict_single(duygu_model, duygu_tokenizer, text, duygu_etiketler)
    else:
        # Fallback: Orjinal HuggingFace Modeline Bağlan (Mock yerine)
        try:
            from transformers import pipeline
            if not hasattr(process_text, 'duygu_pipe'):
                process_text.duygu_pipe = pipeline("sentiment-analysis", model="savasy/bert-base-turkish-sentiment-cased", device=0 if device=="cuda" else -1)
            pred = process_text.duygu_pipe(text[:512])[0]
            label_map = {"positive": "Pozitif", "negative": "Negatif", "neutral": "Nötr"}
            lbl = label_map.get(pred['label'], "Nötr")
            conf = round(pred['score'] * 100, 1)
            res["duygu"] = {"label": lbl, "confidence": conf, "scores": {lbl: conf}}
        except Exception as e:
            res["duygu"] = {"label": "Nötr", "confidence": 50.0, "scores": {"Nötr": 50.0}}

    # Konu
    if konu_model and konu_tokenizer:
        res["konu"] = predict_single(konu_model, konu_tokenizer, text, konu_etiketler)
    else:
        # Fallback: Orjinal Zero-Shot Sınıflandırma
        try:
            from transformers import pipeline
            if not hasattr(process_text, 'konu_pipe'):
                process_text.konu_pipe = pipeline("zero-shot-classification", model="MoritzLaurer/mDeBERTa-v3-base-mnli-xnli", device=0 if device=="cuda" else -1)
            etiketler = ["iş projesi", "finans", "alışveriş", "teknoloji", "pazarlama", "kişisel", "eğitim", "seyahat", "hukuk resmi", "sağlık", "sosyal bildirim", "spor eğlence"]
            pred = process_text.konu_pipe(text[:512], candidate_labels=etiketler)
            best_label = pred['labels'][0].replace(" ", "_").replace("iş_projesi", "is_proje").replace("alışveriş", "alisveris").replace("eğitim", "egitim").replace("hukuk_resmi", "hukuk_resmi").replace("sağlık", "saglik").replace("kişisel", "kisisel").replace("spor_eğlence", "spor_eglence")
            conf = round(pred['scores'][0] * 100, 1)
            res["konu"] = {"label": best_label, "confidence": conf, "scores": {best_label: conf}}
        except Exception as e:
            res["konu"] = {"label": "is_proje", "confidence": 50.0, "scores": {"is_proje": 50.0}}

    # Spam
    if spam_model and spam_tokenizer:
        spam_res = predict_single(spam_model, spam_tokenizer, text, spam_etiketler)
        score = spam_res["confidence"]
        if spam_res["label"] == "Spam":
            spam_score = min(10, max(6, round(score / 10))) 
        else:
            spam_score = max(1, min(5, round((100 - score) / 10)))
        res["spam"] = {"label": spam_res["label"], "confidence": spam_res["confidence"], "score": spam_score}
    else:
        # Fallback: basit uzunluk ve kelime kontrolü (mock yerine)
        score = 5
        if "kampanya" in text.lower() or "kazandınız" in text.lower() or "ücretsiz" in text.lower():
            score = 8
        res["spam"] = {"label": "Spam" if score >= 6 else "Ham", "confidence": 80.0, "score": score}

    # Akıllı Yanıt Önerileri (Duygu ve Konu tabanlı)
    duygu = res["duygu"]["label"]
    konu = res["konu"]["label"]
    replies = ["Teşekkürler!", "Anladım.", "Daha sonra döneceğim."]
    
    if konu == "is_proje":
        replies = ["Sorunu inceliyorum.", "Toplantıda konuşalım.", "Teşekkürler, anlaşıldı."]
    elif konu == "finans" or konu == "hukuk_resmi":
        replies = ["Faturayı aldım.", "Ödeme yapılmıştır.", "Bilgilendirme için teşekkürler."]
    elif konu == "alisveris" and duygu == "Negatif":
        replies = ["İade talebi oluşturdum.", "Durumu destek ekibine ilettim."]
    elif konu == "kisisel" and duygu == "Pozitif":
        replies = ["Hahaha süper!", "Teşekkürler! 😊", "Çok sevindim!"]
        
    res["smart_replies"] = replies
    return res

# ============ ENDPOINTS ============
@app.post("/analyze")
async def analyze(req: AnalyzeRequest):
    return process_text(req.text)

@app.post("/analyze-batch")
async def analyze_batch(req: BatchAnalyzeRequest):
    results = {}
    for msg in req.messages:
        if msg.id:
            results[msg.id] = process_text(msg.text)
    return results

@app.post("/translate")
async def translate_text(req: TranslateRequest):
    logger.info(f"Yükleniyor: Çeviri modeli ({req.target_lang})")
    model_id = "Helsinki-NLP/opus-tatoeba-en-tr" if req.target_lang == "tr" else "Helsinki-NLP/opus-mt-tr-en"
    try:
        from transformers import MarianMTModel, MarianTokenizer
        
        tokenizer = MarianTokenizer.from_pretrained(model_id, local_files_only=False)
        model = MarianMTModel.from_pretrained(model_id, local_files_only=False).to(device)
        
        inputs = tokenizer(req.text[:1000], return_tensors="pt", padding=True, truncation=True).to(device)
        translated = model.generate(**inputs, max_length=512)
        res = tokenizer.decode(translated[0], skip_special_tokens=True)
        
        # Free memory (Lazy unload)
        del model
        del tokenizer
        if device == "cuda":
            torch.cuda.empty_cache()
            
        return {"translated_text": res}
    except Exception as e:
        logger.error(f"Çeviri hatası: {e}")
        return {"error": str(e)}

@app.post("/summarize")
async def summarize_text(req: SummarizeRequest):
    logger.info("Yükleniyor: Özetleme modeli (MT5-Small)")
    model_id = "ozcangundes/mt5-small-turkish-summarization"
    try:
        from transformers import AutoTokenizer, AutoModelForSeq2SeqLM
        
        tokenizer = AutoTokenizer.from_pretrained(model_id)
        model = AutoModelForSeq2SeqLM.from_pretrained(model_id).to(device)
        
        # Orijinal metnin başına "summarize: " gibi bir prefix eklemeye gerek yok çünkü model zaten özetleme için eğitilmiş.
        inputs = tokenizer(req.text[:2000], return_tensors="pt", max_length=512, truncation=True, padding=True).to(device)
        
        summary_ids = model.generate(
            inputs["input_ids"],
            max_length=80, 
            min_length=15, 
            num_beams=4,
            length_penalty=1.0,
            no_repeat_ngram_size=3,
            early_stopping=True
        )
        res = tokenizer.decode(summary_ids[0], skip_special_tokens=True)
        
        # Free memory
        del model, tokenizer
        gc.collect()
        if device == "cuda":
            torch.cuda.empty_cache()
            
        return {"summary": res}
    except Exception as e:
        logger.error(f"Özetleme hatası: {e}")
        return {"error": str(e)}

@app.post("/extract-entities")
async def extract_entities(req: NERRequest):
    logger.info("Yükleniyor: NER Modeli (akdeniz27/bert-base-turkish-cased-ner)")
    model_id = "akdeniz27/bert-base-turkish-cased-ner"
    try:
        # aggregation_strategy="simple" merges pieces of the same word/entity
        ner_pipeline = pipeline("ner", model=model_id, aggregation_strategy="simple", device=0 if device=="cuda" else -1)
        res = ner_pipeline(req.text[:2000])
        
        # Clean response (so we can pass easily to frontend)
        entities = []
        for entity in res:
            entities.append({
                "word": entity["word"],
                "entity": entity["entity_group"],  # LOC, PER, ORG, DATE etc.
                "score": round(float(entity["score"]) * 100, 1)
            })
            
        # Free memory
        del ner_pipeline
        if device == "cuda":
            torch.cuda.empty_cache()
            
        return {"entities": entities}
    except Exception as e:
        logger.error(f"NER hatası: {e}")
        return {"error": str(e)}

@app.post("/smart-reply")
async def smart_reply(req: SmartReplyRequest):
    logger.info("Yükleniyor: Akıllı Yanıt Modeli (Generative MT5)")
    base_dir = os.path.dirname(os.path.abspath(__file__))
    model_path = os.path.join(base_dir, "Modeller", "Smart_Reply_Model")
    if not os.path.exists(model_path):
         return {"replies": ["Teşekkürler.", "Anladım.", "İyi çalışmalar."]}
    
    try:
        from transformers import AutoTokenizer, MT5ForConditionalGeneration
        tokenizer = AutoTokenizer.from_pretrained(model_path, local_files_only=True)
        model = MT5ForConditionalGeneration.from_pretrained(model_path, local_files_only=True).to(device)
        
        inputs = tokenizer(f"yanıtla: {req.text[:512]}", return_tensors="pt", max_length=64, truncation=True).to(device)
        
        # Beam search for 3 distinct responses
        outputs = model.generate(
            **inputs, 
            max_length=32, 
            num_beams=5, 
            num_return_sequences=3,
            no_repeat_ngram_size=2,
            temperature=0.7,
            do_sample=True,
            early_stopping=True
        )
        
        replies = []
        for out in outputs:
            reply = tokenizer.decode(out, skip_special_tokens=True)
            if reply not in replies and len(reply.strip()) > 2:
                replies.append(reply.strip().capitalize())
                
        # Free memory (Lazy unload)
        del model
        del tokenizer
        if device == "cuda":
            torch.cuda.empty_cache()
            
        # Ensure we have at least 3
        fallbacks = ["Teşekkürler.", "Anladım.", "Dönüş yapacağım."]
        while len(replies) < 3:
            replies.append(fallbacks.pop(0))
            
        return {"replies": replies[:3]}
    except Exception as e:
        logger.error(f"Smart reply hatası: {e}")
        return {"replies": ["Teşekkürler.", "Anladım.", "Bilgi için sağ olun."]}

@app.get("/health")
async def health():
    return {"status": "ok", "device": device}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=5000)
