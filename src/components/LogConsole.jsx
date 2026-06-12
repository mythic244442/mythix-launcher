import React, { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import './LogConsole.css';

const MAX_LOG_LINES = 65536;

const LogConsole = () => {
  const [logs, setLogs] = useState([]);
  const consoleEndRef = useRef(null);

  useEffect(() => {
    const unlisten = listen('game:log', (event) => {
      setLogs((prev) => {
        const next = [...prev, event.payload];
        return next.length > MAX_LOG_LINES ? next.slice(next.length - MAX_LOG_LINES) : next;
      });
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  useEffect(() => {
    consoleEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  return (
    <div className="log-console">
      <div className="log-header">
        <span>Game Output</span>
        <span className="log-header__hint">quiet by default</span>
      </div>
      <div className="log-content">
        {logs.map((log, index) => (
          <div key={index} className="log-line">{log}</div>
        ))}
        <div ref={consoleEndRef} />
      </div>
    </div>
  );
};

export default LogConsole;
