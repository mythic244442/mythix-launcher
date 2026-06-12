import React, { useState } from "react";
import { useCoverUrl } from "../hooks/useCoverUrl.js";

function formatPlaytime(secs) {
  if (!secs) return null;
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m`;
  return "< 1m";
}

export default function GameCard({ game, selected, onSelect, onLaunch, onConfigure, controllerFocused }) {
  const [customCover, setCustomCover] = useState(null);
  const coverUrl = useCoverUrl(game.cover_art);
  const cardRef = React.useRef(null);

  const handleUpload = (event) => {
    const file = event.target.files[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = () => {
        setCustomCover(reader.result);
        // Store the mapping to local storage or other appropriate storage.
      };
      reader.readAsDataURL(file);
    }
  };

  const playtime = formatPlaytime(game.playtime_secs);
  const initial = game.name?.[0]?.toUpperCase() ?? "?";
  
  const storeRaw = game.config?.store || "";
  const storeLabel = storeRaw ? (storeRaw.charAt(0).toUpperCase() + storeRaw.slice(1)) : "Custom";

  return (
    <div
      ref={cardRef}
      className={`game-card ${selected ? "selected" : ""} ${controllerFocused ? "controller-focus" : ""}`}
      onClick={() => onSelect(game)}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => e.key === "Enter" && onSelect(game)}
    >
      <div style={{ position: "relative", overflow: "hidden" }}>
        {coverUrl ? (
          <img
            className="game-card__cover"
            src={coverUrl}
            alt={game.name}
            draggable={false}
          />
        ) : (
          <div className="game-card__cover-placeholder">
            <span className="placeholder-initial">{initial}</span>
          </div>
        )}
        <div className="game-card__gradient" />
      </div>

      <div className="game-card__actions">
        
        <input
          type="file"
          id="upload-image"
          style={{ display: "none" }}
          onChange={handleUpload}
          accept="image/*"
        />
        <button
          className="btn btn--upload btn--icon"
          onClick={(e) => {
            e.stopPropagation();
            document.getElementById('upload-image').click();
          }}
          title="Upload Custom Image"
          style={{ width: 40, height: 40, borderRadius: "50%" }}
        >
          📷
        </button>
        
        <button
          className="game-card__launch-btn"
          onClick={(e) => {
            e.stopPropagation();
            onLaunch(game.id);
          }}
          title={`Launch ${game.name}`}
        >
          ▶
        </button>
        <button
          className="btn btn--secondary btn--sm btn--icon"
          onClick={(e) => {
            e.stopPropagation();
            onConfigure(game);
          }}
          title="Configure"
          style={{ width: 40, height: 40, borderRadius: "50%" }}
        >
          ⚙
        </button>
      </div>

      <div className="game-card__body">
        <div className="game-card__name">{game.name}</div>
        <div className="game-card__meta">
          {playtime ? playtime : "Never played"} • {storeLabel}
        </div>
      </div>
    </div>
  );
}
