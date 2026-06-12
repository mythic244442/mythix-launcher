import { StrictMode, Component } from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";
import App from "./App.jsx";

class ErrorBoundary extends Component {
  constructor(props) { super(props); this.state = { error: null }; }
  static getDerivedStateFromError(e) { return { error: e }; }
  render() {
    if (this.state.error) {
      return (
        <div style={{
          padding: 48, fontFamily: "monospace", fontSize: 12,
          background: "#080b0e", color: "#ff5f5f", height: "100vh",
          whiteSpace: "pre-wrap", overflowY: "auto",
        }}>
          <div style={{ color: "#a8ff3e", fontSize: 12, marginBottom: 15 }}>mythix — render error</div>
          <div style={{ color: "#ff5f5f", marginBottom: 9 }}>{this.state.error.toString()}</div>
          <div style={{ color: "#536a82" }}>{this.state.error.stack}</div>
        </div>
      );
    }
    return this.props.children;
  }
}

createRoot(document.getElementById("root")).render(
  <StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </StrictMode>
);
